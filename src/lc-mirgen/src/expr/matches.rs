use super::*;
use lc_ast::BinOp;
use lc_index::Idx;
use std::ops::{Deref, DerefMut};

struct PatternBuilder<'a, 'b, 'tcx> {
    builder: &'b mut MirBuilder<'a, 'tcx>,
    /// predicate blocks
    pblocks: Vec<BlockId>,
    body_blocks: Vec<BlockId>,
    /// the lvalue to write the match expressions value into
    dest: Lvalue<'tcx>,
}

impl<'a, 'b, 'tcx> PatternBuilder<'a, 'b, 'tcx> {
    pub fn new(builder: &'b mut MirBuilder<'a, 'tcx>, dest: Lvalue<'tcx>) -> Self {
        Self { builder, dest, pblocks: Default::default(), body_blocks: Default::default() }
    }
}

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
    pub fn build_naive_match(
        &mut self,
        block: BlockId,
        dest: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        scrut: &tir::Expr<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) -> BlockAnd<()> {
        PatternBuilder::new(self, dest).build(block, expr, scrut, arms)
    }
}

impl<'a, 'b, 'tcx> PatternBuilder<'a, 'b, 'tcx> {
    /// translates match expressions into an if-else chain
    pub fn build(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
        scrut: &tir::Expr<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);

        // allocate initial basic blocks for each arm
        arms.iter().for_each(|_arm| {
            let pblock = self.append_basic_block();
            let body_block = self.append_basic_block();
            self.pblocks.push(pblock);
            self.body_blocks.push(body_block);
        });

        let scrut = set!(block = self.as_lvalue(block, scrut));
        let initial_pblock = self.pblocks[0];
        self.branch(info, block, initial_pblock);

        let final_block = self.append_basic_block();

        for i in 0..arms.len() {
            let next_block_opt = self.pblocks.get(i + 1).copied();
            let next_block = next_block_opt.unwrap_or_else(|| self.mk_abort(info));
            let pblock = self.pblocks[i];
            let mut body_block = self.body_blocks[i];
            set!(
                body_block =
                    self.build_match_arm(pblock, body_block, next_block, expr, scrut, &arms[i])
            );
            self.terminate(info, body_block, TerminatorKind::Branch(final_block));
        }

        final_block.unit()
    }

    fn build_match_arm(
        &mut self,
        mut pblock: BlockId,
        mut body_block: BlockId,
        next_block: BlockId,
        expr: &tir::Expr<'tcx>,
        scrut: Lvalue<'tcx>,
        arm: &tir::Arm<'tcx>,
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        let tcx = self.tcx;
        let dest = self.dest;
        // if `predicate` is true, then its corresponding branch will be executed
        let predicate = self.alloc_tmp(info, tcx.types.bool).into();
        // predicate starts off as true by default
        let b = tcx.mk_const_bool(true);
        self.push_assignment(info, pblock, predicate, Rvalue::Operand(Operand::Const(b)));
        set!(pblock = self.build_arm_predicate(pblock, predicate, scrut, &arm.pat));
        self.terminate(
            info,
            pblock,
            TerminatorKind::Cond(Operand::Lvalue(predicate), body_block, next_block),
        );
        // the next statement must come after the termination statement above
        // as we wish to branch to the start of the `body_block`
        set!(body_block = self.write_expr(body_block, dest, &arm.body));
        body_block.unit()
    }

    /// builds to code to test whether an arm's pattern matches
    // BIG TODO clean this up
    fn build_arm_predicate(
        &mut self,
        mut pblock: BlockId,
        predicate: Lvalue<'tcx>,
        scrut: Lvalue<'tcx>,
        pat: &tir::Pattern<'tcx>,
    ) -> BlockAnd<Lvalue<'tcx>> {
        let tcx = self.tcx;
        let info = self.span_info(pat.span);
        match pat.kind {
            tir::PatternKind::Wildcard => {}
            tir::PatternKind::Box(ref pat) => {
                let deref_scrut = tcx.project_deref(scrut);
                set!(pblock = self.build_arm_predicate(pblock, predicate, deref_scrut, pat));
            }
            tir::PatternKind::Binding(_m, _ident, ref sub) => {
                assert!(sub.is_none());
                set!(pblock = self.bind_pat_to_lvalue(pblock, pat, scrut));
            }
            tir::PatternKind::Field(ref pats) =>
                for tir::FieldPat { index, pat } in pats {
                    set!(
                        pblock = self.build_arm_predicate(
                            pblock,
                            predicate,
                            tcx.project_field(scrut, *index, pat.ty),
                            pat
                        )
                    );
                },
            tir::PatternKind::Lit(c) => {
                // compare the literal expression with the scrutinee
                let cmp_rvalue = set!(
                    pblock = self.build_binary_op(
                        pblock,
                        pat.span,
                        tcx.types.bool,
                        BinOp::Eq,
                        Operand::Const(c),
                        Operand::Lvalue(scrut),
                    )
                );
                let cmp_lvalue = self.alloc_tmp(info, tcx.types.bool).into();
                self.push_assignment(info, pblock, cmp_lvalue, cmp_rvalue);
                // TODO factor this out somehow
                // `and` the predicate
                let and = set!(
                    pblock = self.build_binary_op(
                        pblock,
                        pat.span,
                        tcx.types.bool,
                        BinOp::And,
                        Operand::Lvalue(cmp_lvalue),
                        Operand::Lvalue(predicate),
                    )
                );
                self.push_assignment(info, pblock, predicate, and);
            }
            tir::PatternKind::Variant(adt, substs, idx, ref pats) => {
                let discriminant_lvalue = self.alloc_tmp(info, tcx.types.discr).into();
                self.push_assignment(
                    info,
                    pblock,
                    discriminant_lvalue,
                    Rvalue::Discriminant(scrut),
                );
                // recall `idx` is the discriminant
                // so we compare this with the discriminant of the scrutinee
                assert!(idx.index() < i16::MAX as usize);
                let discr = tcx.mk_const_discr(idx.index() as i16);
                let cmp_rvalue = set!(
                    pblock = self.build_binary_op(
                        pblock,
                        pat.span,
                        tcx.types.bool,
                        BinOp::Eq,
                        Operand::Const(discr),
                        Operand::Lvalue(discriminant_lvalue),
                    )
                );

                // TODO factor this out somehow
                // `and` the predicate
                let cmp_lvalue = self.alloc_tmp(info, tcx.types.bool).into();
                self.push_assignment(info, pblock, cmp_lvalue, cmp_rvalue);
                let and = set!(
                    pblock = self.build_binary_op(
                        pblock,
                        pat.span,
                        tcx.types.bool,
                        BinOp::And,
                        Operand::Lvalue(cmp_lvalue),
                        Operand::Lvalue(predicate),
                    )
                );
                self.push_assignment(info, pblock, predicate, and);

                // project past the discriminant into the enum content
                let variant_ty =
                    tcx.mk_tup_iter(adt.variants[idx].fields.iter().map(|f| f.ty(tcx, substs)));
                let enum_content_lvalue = LvalueBuilder::from(scrut)
                    .project_field(FieldIdx::new(1), tcx.mk_adt_ty(adt, substs))
                    .project_cast(variant_ty)
                    .lvalue(tcx);
                for (i, pat) in pats.iter().enumerate() {
                    set!(
                        pblock = self.build_arm_predicate(
                            pblock,
                            predicate,
                            tcx.project_field(enum_content_lvalue, FieldIdx::new(i), pat.ty),
                            pat
                        )
                    );
                }
            }
        };
        pblock.and(predicate)
    }
}

impl<'a, 'b, 'tcx> Deref for PatternBuilder<'a, 'b, 'tcx> {
    type Target = MirBuilder<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        self.builder
    }
}

impl<'a, 'b, 'tcx> DerefMut for PatternBuilder<'a, 'b, 'tcx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.builder
    }
}

use super::{BlockAnd, Builder};
use crate::ast::BinOp;
use crate::mir::*;
use crate::set;
use crate::tir;
use crate::ty::TyKind;
use crate::typeck::TyCtx;
use std::ops::{Deref, DerefMut};

struct PatternBuilder<'a, 'b, 'tcx> {
    builder: &'b mut Builder<'a, 'tcx>,
    /// predicate blocks
    pblocks: Vec<BlockId>,
    body_blocks: Vec<BlockId>,
}

impl<'a, 'b, 'tcx> PatternBuilder<'a, 'b, 'tcx> {
    pub fn new(builder: &'b mut Builder<'a, 'tcx>) -> Self {
        Self { builder, pblocks: Default::default(), body_blocks: Default::default() }
    }
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn build_naive_match(
        &mut self,
        block: BlockId,
        dest: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        scrut: &tir::Expr<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) -> BlockAnd<()> {
        PatternBuilder::new(self).build(block, dest, expr, scrut, arms)
    }
}

impl<'a, 'b, 'tcx> PatternBuilder<'a, 'b, 'tcx> {
    /// translates match expressions into an if-else chain
    pub fn build(
        &mut self,
        mut block: BlockId,
        dest: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        scrut: &tir::Expr<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);

        // allocate initial basic blocks for each arm
        arms.iter().for_each(|arm| {
            let pblock = self.append_basic_block();
            let body_block = self.append_basic_block();
            self.pblocks.push(pblock);
            self.body_blocks.push(body_block);
        });

        let scrut = set!(block = self.as_lvalue(block, scrut));
        let initial_block = self.pblocks[0];
        self.branch(info, block, initial_block);

        let final_block = self.append_basic_block();

        for i in 0..arms.len() {
            let next_block_opt = self.pblocks.get(i + 1).copied();
            let next_block = next_block_opt.unwrap_or_else(|| self.mk_unreachable(info));
            let pblock = self.pblocks[i];
            let mut body_block = self.body_blocks[i];
            set!(
                body_block = self
                    .build_match_arm(pblock, body_block, next_block, dest, expr, scrut, &arms[i])
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
        dest: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        scrut: Lvalue<'tcx>,
        arm: &tir::Arm<'tcx>,
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        let predicate = set!(pblock = self.build_arm_predicate(pblock, scrut, arm.pat));
        set!(body_block = self.write_expr(body_block, dest, arm.body));
        self.terminate(
            info,
            pblock,
            TerminatorKind::Cond(Operand::Lvalue(predicate), body_block, next_block),
        );
        body_block.unit()
    }

    /// builds to code to test whether an arm's pattern matches
    fn build_arm_predicate(
        &mut self,
        mut pblock: BlockId,
        scrut: Lvalue<'tcx>,
        pat: &tir::Pattern<'tcx>,
    ) -> BlockAnd<Lvalue<'tcx>> {
        let tcx = self.tcx;
        let info = self.span_info(pat.span);
        // if `predicate` is true, then its corresponding branch will be executed
        let predicate = self.alloc_tmp(info, tcx.types.boolean).into();
        // predicate starts off as true by default
        let b = self.mk_const_bool(true);
        self.push_assignment(info, pblock, predicate, Rvalue::Operand(Operand::Const(b)));
        match pat.kind {
            tir::PatternKind::Wildcard => {}
            tir::PatternKind::Binding(m, ident, sub) => {
                assert!(sub.is_none());
                // TODO bind the names
            }
            tir::PatternKind::Field(_) => todo!(),
            tir::PatternKind::Lit(expr) => {
                let tmp = self.alloc_tmp(info, expr.ty).into();
                set!(pblock = self.write_expr(pblock, tmp, expr));
                // compare the literal expression with the scrutinee
                let cmp_rvalue = set!(
                    pblock = self.build_binary_op(
                        pblock,
                        pat.span,
                        tcx.types.boolean,
                        BinOp::Eq,
                        Operand::Lvalue(tmp),
                        Operand::Lvalue(scrut),
                    )
                );
                let cmp_lvalue = self.alloc_tmp(info, tcx.types.boolean).into();
                self.push_assignment(info, pblock, cmp_lvalue, cmp_rvalue);
                // `and` the predicate
                let and = set!(
                    pblock = self.build_binary_op(
                        pblock,
                        pat.span,
                        tcx.types.boolean,
                        BinOp::And,
                        Operand::Lvalue(cmp_lvalue),
                        Operand::Lvalue(predicate),
                    )
                );
                self.push_assignment(info, pblock, predicate, and);
            }
            tir::PatternKind::Variant(_, _, _) => todo!(),
        };
        pblock.and(predicate)
    }
}

impl<'a, 'b, 'tcx> Deref for PatternBuilder<'a, 'b, 'tcx> {
    type Target = Builder<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl<'a, 'b, 'tcx> DerefMut for PatternBuilder<'a, 'b, 'tcx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}
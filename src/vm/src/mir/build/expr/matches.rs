use super::{BlockAnd, Builder};
use crate::ast::BinOp;
use crate::mir::*;
use crate::set;
use crate::tir;
use crate::ty::TyKind;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    /// translates match expressions into a large if-else chain
    pub fn build_naive_match(
        &mut self,
        block: BlockId,
        dest: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        scrut: &tir::Expr<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) -> BlockAnd<()> {
        let mut pblocks = vec![];
        let mut body_blocks = vec![];
        for arm in arms {
            pblocks.push(self.append_basic_block());
            body_blocks.push(self.append_basic_block());
        }
        todo!()
    }

    fn build_match_arm(
        &mut self,
        block: BlockId,
        dest: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        scrut: &tir::Expr<'tcx>,
        arm: &tir::Arm<'tcx>,
    ) -> BlockAnd<()> {
        todo!()
    }

    /// builds to code to test whether an arm's pattern matches
    fn build_arm_predicate(
        &mut self,
        mut block: BlockId,
        scrut: Lvalue<'tcx>,
        pat: tir::Pattern<'tcx>,
    ) {
        let info = self.span_info(pat.span);
        // if `predicate` is true, then its corresponding branch will be executed
        let predicate = self.alloc_tmp(info, self.tcx.types.boolean).into();
        // predicate starts off as true by default
        let b = self.mk_const_bool(true);
        self.push_assignment(info, block, predicate, Rvalue::Operand(Operand::Const(b)));
        match pat.kind {
            tir::PatternKind::Wildcard => {}
            tir::PatternKind::Binding(m, ident, sub) => {
                assert!(sub.is_none());
                // TODO bind the names
            }
            tir::PatternKind::Field(_) => todo!(),
            tir::PatternKind::Lit(expr) => {
                let tmp = self.alloc_tmp(info, expr.ty).into();
                set!(block = self.write_expr(block, tmp, expr));
                // compare the literal expression with the scrutinee
                let cmp = self.build_binary_op(
                    block,
                    pat.span,
                    self.tcx.types.boolean,
                    BinOp::Eq,
                    Operand::Lvalue(tmp),
                    Operand::Lvalue(scrut),
                );
                let cmp_rvalue = self.alloc_tmp(info, self.tcx.types.boolean).into();
                // `and` the predicate
                let and = set!(
                    block = self.build_binary_op(
                        block,
                        pat.span,
                        self.tcx.types.boolean,
                        BinOp::And,
                        Operand::Lvalue(cmp_rvalue),
                        Operand::Lvalue(predicate),
                    )
                );
                self.push_assignment(info, block, predicate, and);
            }
            tir::PatternKind::Variant(_, _, _) => {}
        }
    }
}

use super::{BlockAnd, BlockAndExt, Builder};
use crate::mir::{BlockId, Lvalue, Operand, Rvalue, TerminatorKind};
use crate::set;
use crate::tir;
use itertools::Itertools;

mod constant;
mod lvalue;
mod operand;
mod rvalue;
mod tmp;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    /// writes `expr` into `lvalue`
    pub fn write_expr(
        &mut self,
        mut block: BlockId,
        lvalue: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        match expr.kind {
            tir::ExprKind::Block(ir) => self.ir_block(block, lvalue, expr, ir),
            tir::ExprKind::ItemRef(_) => todo!(),
            tir::ExprKind::Tuple(_) => todo!(),
            tir::ExprKind::Lambda(_) => todo!(),
            tir::ExprKind::Call(_, _) => todo!(),
            tir::ExprKind::Match(scrut, arms) => self.build_match(block, expr, lvalue, scrut, arms),
            tir::ExprKind::Assign(lhs, rhs) => {
                let rvalue = set!(block = self.as_rvalue(block, rhs));
                let lvalue = set!(block = self.as_lvalue(block, lhs));
                self.cfg.push_assignment(info, block, lvalue, rvalue);
                block.unit()
            }
            tir::ExprKind::VarRef(..)
            | tir::ExprKind::Unary(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Const(..) => {
                let rvalue = set!(block = self.as_rvalue(block, expr));
                self.cfg.push_assignment(info, block, lvalue, rvalue);
                block.unit()
            }
            tir::ExprKind::Ret(_) => self.build_expr_stmt(block, expr),
        }
    }

    fn build_match(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
        lvalue: Lvalue<'tcx>,
        scrut: &tir::Expr<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        let scrut_rvalue = set!(block = self.as_rvalue(block, scrut));
        // TODO
        let arm_blocks = arms
            .iter()
            .map(|arm| self.build_arm(lvalue, &scrut_rvalue, arm).unpack())
            .collect_vec();
        let end = self.cfg.append_basic_block();
        arm_blocks
            .iter()
            .for_each(|&(b, _)| self.cfg.terminate(info, b, TerminatorKind::Branch(end)));
        self.cfg.terminate(
            info,
            block,
            TerminatorKind::Switch { discr: scrut_rvalue, arms: arm_blocks, default: None },
        );
        end.unit()
    }

    fn build_arm(
        &mut self,
        dest: Lvalue<'tcx>,
        scrut_rvalue: &Rvalue<'tcx>,
        arm: &tir::Arm<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        let mut block = self.cfg.append_basic_block();
        set!(block = self.write_expr(block, dest, arm.body));
        let rvalue = set!(block = self.build_arm_pat(block, arm.pat, scrut_rvalue));
        block.and(rvalue)
    }

    fn build_arm_pat(
        &mut self,
        block: BlockId,
        pat: &tir::Pattern<'tcx>,
        cmp_rval: &Rvalue<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        match pat.kind {
            tir::PatternKind::Wildcard => block.and(cmp_rval.clone()),
            tir::PatternKind::Binding(ident, _) => {
                self.alloc_local(pat);
                block.and(cmp_rval.clone())
            }
            tir::PatternKind::Field(_) => todo!(),
            tir::PatternKind::Lit(c) => self.as_rvalue(block, c),
        }
    }

    fn ir_block(
        &mut self,
        mut block: BlockId,
        lvalue: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        ir: &tir::Block<'tcx>,
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        for stmt in ir.stmts {
            set!(block = self.build_stmt(block, stmt));
        }

        if let Some(expr) = ir.expr {
            set!(block = self.write_expr(block, lvalue, expr));
        } else {
            self.cfg.push_unit(info, block, lvalue)
        }
        block.unit()
    }
}

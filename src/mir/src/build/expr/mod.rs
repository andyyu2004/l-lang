use super::*;
use crate::set;
use ir::FieldIdx;
use itertools::Itertools;
use lcore::mir::*;

use lcore::ty::Projection;
pub use lvalue::LvalueBuilder;

mod closure;
mod constant;
mod lvalue;
mod matches;
mod operand;
mod rvalue;
mod tmp;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    /// writes `expr` into `dest`
    // this method exists as it is easier to implement certain expressions
    // given an `lvalue` to write the result of the expression into
    // opposed to returning an rvalue directly
    pub fn write_expr(
        &mut self,
        mut block: BlockId,
        dest: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<()> {
        // some issues with differences in mutability currently
        // debug_assert_eq!(self.lvalue_ty(dest), expr.ty);
        let info = self.span_info(expr.span);
        match &expr.kind {
            tir::ExprKind::Block(ir) => self.build_ir_block(block, dest, expr, ir),
            tir::ExprKind::Call(f, args) => self.build_call(block, dest, expr, f, args),
            tir::ExprKind::Match(scrut, arms) => {
                self.build_naive_match(block, dest, expr, &scrut, &arms)
                // self.build_match(block, dest, expr, scrut, arms)
            }
            tir::ExprKind::Box(inner) => {
                self.push_assignment(info, block, dest, Rvalue::Box(inner.ty));
                // write the `inner` expression into the allocated memory
                set!(block = self.write_expr(block, self.tcx.project_deref(dest), &inner));
                block.unit()
            }
            tir::ExprKind::Ret(_) => self.build_expr_stmt(block, expr),
            tir::ExprKind::Tuple(xs) => self.build_tuple(block, dest, expr, &xs),
            tir::ExprKind::VarRef(..)
            | tir::ExprKind::ItemRef(..)
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Closure { .. }
            | tir::ExprKind::Ref(..)
            | tir::ExprKind::Field(..)
            | tir::ExprKind::Assign(..)
            | tir::ExprKind::Unary(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Deref(..)
            | tir::ExprKind::Const(..) => {
                let rvalue = set!(block = self.as_rvalue(block, expr));
                self.push_assignment(info, block, dest, rvalue);
                block.unit()
            }
        }
    }

    fn build_tuple(
        &mut self,
        mut block: BlockId,
        dest: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        xs: &[tir::Expr<'tcx>],
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        xs.iter().enumerate().for_each(|(i, x)| {
            let rvalue = set!(block = self.as_rvalue(block, x));
            let lvalue = self.tcx.project_lvalue(dest, Projection::Field(FieldIdx::new(i), x.ty));
            self.push_assignment(info, block, lvalue, rvalue);
        });
        block.unit()
    }

    fn build_call(
        &mut self,
        mut block: BlockId,
        lvalue: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        f: &tir::Expr<'tcx>,
        args: &[tir::Expr<'tcx>],
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        let f = set!(block = self.as_operand(block, f));
        let args = args.iter().map(|arg| set!(block = self.as_operand(block, arg))).collect_vec();
        let target = self.append_basic_block();
        self.terminate(info, block, TerminatorKind::Call { f, args, lvalue, target, unwind: None });
        target.unit()
    }

    fn build_match(
        &mut self,
        mut block: BlockId,
        lvalue: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        scrut: &tir::Expr<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        let scrut_operand = set!(block = self.as_operand(block, scrut));

        // terminate all the switch blocks to branch back together again
        let end_block = self.append_basic_block();
        let (arm_blocks, default) = self.build_arms(info, lvalue, &scrut_operand, arms, end_block);

        // if there is no default block, just create an abort block (as we don't check for exhaustive patterns)
        let default = default.unwrap_or_else(|| self.mk_abort(info));

        self.terminate(
            info,
            block,
            TerminatorKind::Switch { discr: scrut_operand, arms: arm_blocks, default },
        );

        end_block.unit()
    }

    fn mk_abort(&mut self, info: SpanInfo) -> BlockId {
        let block = self.append_basic_block();
        self.terminate(info, block, TerminatorKind::Abort);
        block
    }

    /// returns the switch arms and maybe the default block
    fn build_arms(
        &mut self,
        info: SpanInfo,
        dest: Lvalue<'tcx>,
        scrut_operand: &Operand<'tcx>,
        arms: &[tir::Arm<'tcx>],
        end_block: BlockId,
    ) -> (Vec<(Operand<'tcx>, BlockId)>, Option<BlockId>) {
        let mut switch_arms = Vec::with_capacity(arms.len());
        for arm in arms {
            // create the block for the body of the arm
            let start_block = self.append_basic_block();
            // we need to two block pointers as we need to terminate the end block,
            // but we need to return the start block for the switch terminator
            let mut block = start_block;
            // the first irrefutable pattern will be assigned the default block of the switch
            let operand = set!(block = self.build_arm_pat(block, &arm.pat, scrut_operand));
            set!(block = self.write_expr(block, dest, &arm.body));
            self.terminate(info, block, TerminatorKind::Branch(end_block));
            if !arm.pat.is_refutable() {
                return (switch_arms, Some(start_block));
            }
            switch_arms.push((operand, start_block));
        }
        (switch_arms, None)
    }

    fn build_arm_pat(
        &mut self,
        block: BlockId,
        pat: &tir::Pattern<'tcx>,
        cmp_operand: &Operand<'tcx>,
    ) -> BlockAnd<Operand<'tcx>> {
        match &pat.kind {
            tir::PatternKind::Wildcard => block.and(cmp_operand.clone()),
            tir::PatternKind::Binding(m, ident, _) => {
                let &tir::Pattern { id, span, ty, .. } = pat;
                self.alloc_local(id, span, ty);
                block.and(cmp_operand.clone())
            }
            tir::PatternKind::Field(_) => todo!(),
            tir::PatternKind::Lit(expr) => self.as_operand(block, &expr),
            tir::PatternKind::Variant(adt, substs, idx, pats) => todo!(),
        }
    }

    fn build_ir_block(
        &mut self,
        mut block: BlockId,
        lvalue: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        ir: &tir::Block<'tcx>,
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        self.with_scope(info, |builder| {
            for stmt in &ir.stmts {
                set!(block = builder.build_stmt(block, stmt));
            }

            if let Some(expr) = &ir.expr {
                set!(block = builder.write_expr(block, lvalue, expr));
            } else {
                builder.push_assign_unit(info, block, lvalue)
            }
            block.unit()
        })
    }
}

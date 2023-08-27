use crate::set;
use crate::*;
use ir::FieldIdx;
use itertools::Itertools;
use lc_core::mir::*;

use lc_core::ty::Projection;
pub use lvalue::LvalueBuilder;

mod closure;
mod constant;
mod lvalue;
mod matches;
mod operand;
mod rvalue;
mod tmp;

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
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
            tir::ExprKind::Loop(ir) => self.build_loop(block, dest, expr, ir),
            tir::ExprKind::Call(f, args) => self.build_call(block, dest, expr, f, args),
            tir::ExprKind::Match(scrut, arms) => {
                self.build_naive_match(block, dest, expr, scrut, arms)
                // self.build_match(block, dest, expr, scrut, arms)
            }
            // these expressions have `!` type, so have no return value so we can build them as a
            // expression statement
            tir::ExprKind::Ret(..) | tir::ExprKind::Break | tir::ExprKind::Continue =>
                self.build_expr_stmt(block, expr),
            tir::ExprKind::Tuple(xs) => self.build_tuple(block, dest, expr, xs),
            tir::ExprKind::Box(..)
            | tir::ExprKind::VarRef(..)
            | tir::ExprKind::Ref(..)
            | tir::ExprKind::ItemRef(..)
            | tir::ExprKind::Field(..)
            | tir::ExprKind::Assign(..)
            | tir::ExprKind::Unary(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Deref(..)
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Closure { .. }
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

    fn mk_abort(&mut self, info: SpanInfo) -> BlockId {
        let block = self.append_basic_block();
        self.terminate(info, block, TerminatorKind::Abort);
        block
    }

    /// fn f() {
    ///     loop {
    ///         ...
    ///     }
    /// }
    ///
    /// f:
    ///   br loop
    ///
    /// loop:
    ///   ...
    ///   br loop
    ///
    /// next:
    ///
    ///
    fn build_loop(
        &mut self,
        prev: BlockId,
        lvalue: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        ir: &tir::Block<'tcx>,
    ) -> BlockAnd<()> {
        let next = self.append_basic_block();
        let loop_start = self.append_basic_block();
        self.with_breakable_scope(expr.span, loop_start, next, |this| {
            let info = this.span_info(expr.span);
            this.branch(info, prev, loop_start);
            let loop_end = set!(this.build_ir_block(loop_start, lvalue, expr, ir));
            this.branch(info, loop_end, loop_start);
            this.append_basic_block().unit()
        })
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

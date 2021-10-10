use super::*;
use lc_core::mir::BlockId;
use lc_core::ty::Const;

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
    pub fn as_const(
        &mut self,
        block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<&'tcx Const<'tcx>> {
        match expr.kind {
            tir::ExprKind::Const(c) => block.and(c),
            tir::ExprKind::Box(..)
            | tir::ExprKind::Unary(..)
            | tir::ExprKind::Deref(..)
            | tir::ExprKind::Ref(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Field(..)
            | tir::ExprKind::Block(..)
            | tir::ExprKind::VarRef(..)
            | tir::ExprKind::ItemRef(..)
            | tir::ExprKind::Tuple(..)
            | tir::ExprKind::Call(..)
            | tir::ExprKind::Match(..)
            | tir::ExprKind::Ret(..)
            | tir::ExprKind::Assign(..)
            | tir::ExprKind::Loop(..)
            | tir::ExprKind::Break
            | tir::ExprKind::Continue
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Closure { .. } => panic!("not a constant"),
        }
    }
}

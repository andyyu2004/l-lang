use super::*;
use lcore::mir::BlockId;
use lcore::ty::Const;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn as_const(
        &mut self,
        block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<&'tcx Const<'tcx>> {
        match expr.kind {
            tir::ExprKind::Const(c) => block.and(c),
            tir::ExprKind::Unary(_, _)
            | tir::ExprKind::Deref(..)
            | tir::ExprKind::Ref(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Box(..)
            | tir::ExprKind::Field(..)
            | tir::ExprKind::Block(..)
            | tir::ExprKind::VarRef(..)
            | tir::ExprKind::ItemRef(..)
            | tir::ExprKind::Tuple(..)
            | tir::ExprKind::Call(..)
            | tir::ExprKind::Match(..)
            | tir::ExprKind::Ret(..)
            | tir::ExprKind::Assign(..)
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Closure { .. } => panic!("not a constant"),
        }
    }
}

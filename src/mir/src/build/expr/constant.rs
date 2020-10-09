use super::*;
use lcore::mir::BlockId;
use lcore::ty::{Const, ConstKind};

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn mk_const_int(&self, i: i64) -> &'tcx Const<'tcx> {
        self.tcx.intern_const(Const::new(ConstKind::Int(i), self.tcx.types.int))
    }

    pub fn mk_const_bool(&self, b: bool) -> &'tcx Const<'tcx> {
        self.tcx.intern_const(Const::new(ConstKind::Bool(b), self.tcx.types.int))
    }

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
            | tir::ExprKind::Block(_)
            | tir::ExprKind::VarRef(_)
            | tir::ExprKind::InstanceRef(_)
            | tir::ExprKind::Tuple(_)
            | tir::ExprKind::Call(_, _)
            | tir::ExprKind::Match(_, _)
            | tir::ExprKind::Ret(..)
            | tir::ExprKind::Assign(_, _)
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Closure { .. } => panic!("not a constant"),
        }
    }
}

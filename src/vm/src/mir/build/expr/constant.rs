use crate::ast;
use crate::mir::build::*;
use crate::mir::*;
use crate::set;
use crate::span::Span;
use crate::ty::{ConstKind, Ty};

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn mk_const_int(&mut self, i: i64) -> &'tcx Const<'tcx> {
        self.tcx.intern_const(Const::new(ConstKind::Int(i)))
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
            | tir::ExprKind::ItemRef(_)
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

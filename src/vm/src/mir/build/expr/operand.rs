use crate::ast;
use crate::mir::build::*;
use crate::mir::*;
use crate::span::Span;
use crate::ty::Ty;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn as_operand(
        &mut self,
        mut block: BlockId,
        expr: &'tcx tir::Expr<'tcx>,
    ) -> BlockAnd<Operand<'tcx>> {
        match expr.kind {
            tir::ExprKind::Const(c) => todo!(),
            tir::ExprKind::Bin(_, _, _) => todo!(),
            tir::ExprKind::Unary(_, _) => todo!(),
            tir::ExprKind::Block(_) => todo!(),
            tir::ExprKind::VarRef(_) => todo!(),
            tir::ExprKind::ItemRef(_) => todo!(),
            tir::ExprKind::Tuple(_) => todo!(),
            tir::ExprKind::Lambda(_) => todo!(),
            tir::ExprKind::Call(_, _) => todo!(),
            tir::ExprKind::Match(_, _) => todo!(),
            tir::ExprKind::Assign(_, _) => todo!(),
            tir::ExprKind::Ret(_) => todo!(),
        }
    }
}

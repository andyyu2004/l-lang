use crate::ast;
use crate::mir::build::*;
use crate::mir::*;
use crate::set;
use crate::span::Span;
use crate::ty::Ty;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn as_rvalue(
        &mut self,
        mut block: BlockId,
        expr: &'tcx tir::Expr<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        match expr.kind {
            tir::ExprKind::Const(_) => todo!(),
            tir::ExprKind::Bin(op, l, r) => {
                let lhs = set!(block = self.as_operand(block, l));
                let rhs = set!(block = self.as_operand(block, r));
                block.and(Rvalue::Bin(op, lhs, rhs))
            }
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

    pub(super) fn build_binary_op(
        &mut self,
        mut block: BlockId,
        span: Span,
        op: ast::BinOp,
        ty: Ty<'tcx>,
        lhs: Operand<'tcx>,
        rhs: Operand<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        block.and(Rvalue::Bin(op, lhs, rhs))
    }
}

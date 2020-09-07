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
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        let info = self.span_info(expr.span);
        match expr.kind {
            tir::ExprKind::Const(_) => {
                let operand = set!(block = self.as_operand(block, expr));
                block.and(Rvalue::Use(operand))
            }
            tir::ExprKind::Bin(op, l, r) => {
                let lhs = set!(block = self.as_operand(block, l));
                let rhs = set!(block = self.as_operand(block, r));
                self.build_binary_op(block, expr.span, expr.ty, op, lhs, rhs)
            }
            tir::ExprKind::Unary(_, _) => todo!(),
            tir::ExprKind::Block(_) => todo!(),
            tir::ExprKind::ItemRef(_) => todo!(),
            tir::ExprKind::Tuple(_) => todo!(),
            tir::ExprKind::Lambda(_) => todo!(),
            tir::ExprKind::Call(_, _) => todo!(),
            tir::ExprKind::Match(_, _) => todo!(),
            tir::ExprKind::Assign(l, r) => {
                let lhs = set!(block = self.as_lvalue(block, l));
                let rhs = set!(block = self.as_rvalue(block, r));
                self.push_assignment(info, block, lhs, rhs);
                block.and(rhs)
            }
            tir::ExprKind::Ret(_) => todo!(),
            tir::ExprKind::VarRef(_) => {
                let operand = set!(block = self.as_operand(block, expr));
                block.and(Rvalue::Use(operand))
            }
        }
    }

    pub(super) fn build_binary_op(
        &mut self,
        block: BlockId,
        span: Span,
        ty: Ty<'tcx>,
        op: ast::BinOp,
        lhs: Operand<'tcx>,
        rhs: Operand<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        // TODO some checks
        block.and(Rvalue::Bin(op, lhs, rhs))
    }
}

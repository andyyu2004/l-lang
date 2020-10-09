use super::*;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn as_operand(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<Operand<'tcx>> {
        match expr.kind {
            tir::ExprKind::InstanceRef(instance) => block.and(Operand::Instance(instance)),
            tir::ExprKind::Field(..) | tir::ExprKind::Deref(..) | tir::ExprKind::VarRef(..) => {
                let lvalue = set!(block = self.as_lvalue(block, expr));
                block.and(Operand::Lvalue(lvalue))
            }
            tir::ExprKind::Const(c) => {
                let constant = set!(block = self.as_const(block, expr));
                block.and(Operand::Const(constant))
            }
            tir::ExprKind::Unary(..)
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Block(_)
            | tir::ExprKind::Box(..)
            | tir::ExprKind::Closure { .. }
            | tir::ExprKind::Match(..)
            | tir::ExprKind::Ref(..)
            | tir::ExprKind::Assign(..)
            | tir::ExprKind::Ret(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Call(..)
            | tir::ExprKind::Tuple(..) => {
                // create temporary var to hold the result
                let lvalue = set!(block = self.as_tmp(block, expr)).into();
                block.and(Operand::Lvalue(lvalue))
            }
        }
    }
}
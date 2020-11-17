use super::*;

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
    crate fn as_operand(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<Operand<'tcx>> {
        match expr.kind {
            // assign each item reference a unique instance id
            // which is later resolved to a particular instance
            // during monomorphization
            tir::ExprKind::ItemRef(def_id, substs) => block.and(Operand::Item(def_id, substs)),
            tir::ExprKind::Field(..) | tir::ExprKind::Deref(..) | tir::ExprKind::VarRef(..) => {
                let lvalue = set!(block = self.as_lvalue(block, expr));
                block.and(Operand::Lvalue(lvalue))
            }
            tir::ExprKind::Const(..) => {
                let constant = set!(block = self.as_const(block, expr));
                block.and(Operand::Const(constant))
            }
            tir::ExprKind::Box(..)
            | tir::ExprKind::Loop(..)
            | tir::ExprKind::Unary(..)
            | tir::ExprKind::Block(..)
            | tir::ExprKind::Match(..)
            | tir::ExprKind::Ref(..)
            | tir::ExprKind::Assign(..)
            | tir::ExprKind::Ret(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Call(..)
            | tir::ExprKind::Tuple(..)
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Closure { .. }
            | tir::ExprKind::Break
            | tir::ExprKind::Continue => {
                // create temporary var to hold the result
                let lvalue = set!(block = self.as_tmp(block, expr)).into();
                block.and(Operand::Lvalue(lvalue))
            }
        }
    }
}

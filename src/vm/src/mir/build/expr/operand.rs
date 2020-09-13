use crate::ast;
use crate::mir::build::*;
use crate::mir::*;
use crate::set;
use crate::span::Span;
use crate::ty::Ty;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn as_operand(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<Operand<'tcx>> {
        match expr.kind {
            tir::ExprKind::Field(..) | tir::ExprKind::Deref(..) => {
                let lvalue = set!(block = self.as_lvalue(block, expr));
                block.and(Operand::Ref(lvalue))
            }
            tir::ExprKind::Const(c) => {
                let constant = set!(block = self.as_const(block, expr));
                block.and(Operand::Const(constant))
            }
            tir::ExprKind::VarRef(id) => {
                let var = self.var_ir_map[&id];
                block.and(Operand::Ref(Lvalue::from(var)))
            }
            tir::ExprKind::ItemRef(def) => block.and(Operand::Item(def)),
            tir::ExprKind::Unary(..)
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Block(_)
            | tir::ExprKind::Box(..)
            | tir::ExprKind::Lambda(..)
            | tir::ExprKind::Match(..)
            | tir::ExprKind::Assign(..)
            | tir::ExprKind::Ret(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Call(..)
            | tir::ExprKind::Tuple(..) => {
                // create temporary var to hold the result
                let lvalue = set!(block = self.as_tmp(block, expr)).into();
                block.and(Operand::Ref(lvalue))
            }
        }
    }
}

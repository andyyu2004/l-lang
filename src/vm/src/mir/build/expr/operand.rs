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
            tir::ExprKind::Adt { .. } => todo!(),
            tir::ExprKind::Box(..) => todo!(),
            tir::ExprKind::Const(c) => {
                let constant = set!(block = self.as_const(block, expr));
                block.and(Operand::Const(constant))
            }
            tir::ExprKind::VarRef(id) => {
                let var = self.var_ir_map[&id];
                block.and(Operand::Ref(Lvalue::from(var)))
            }
            tir::ExprKind::ItemRef(def) => block.and(Operand::Item(def)),
            tir::ExprKind::Field(..) => {
                let lvalue = set!(block = self.as_lvalue(block, expr));
                block.and(Operand::Ref(lvalue))
            }
            tir::ExprKind::Unary(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Call(..)
            | tir::ExprKind::Tuple(..) => {
                // create temporary var to hold the result
                let lvalue = set!(block = self.as_tmp(block, expr)).into();
                block.and(Operand::Ref(lvalue))
            }
            tir::ExprKind::Block(_) => todo!(),
            tir::ExprKind::Lambda(_) => todo!(),
            tir::ExprKind::Match(_, _) => todo!(),
            tir::ExprKind::Assign(_, _) => todo!(),
            tir::ExprKind::Ret(_) => todo!(),
        }
    }
}

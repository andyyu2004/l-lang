use crate::ast;
use crate::mir::build::*;
use crate::mir::*;
use crate::set;
use crate::span::Span;
use crate::ty::Ty;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn as_lvalue(&mut self, block: BlockId, expr: &tir::Expr<'tcx>) -> BlockAnd<Lvalue<'tcx>> {
        match expr.kind {
            tir::ExprKind::VarRef(id) => block.and(Lvalue::from(self.var_ir_map[&id])),
            tir::ExprKind::Const(_) => unreachable!(),
            tir::ExprKind::Bin(_, _, _) => unreachable!(),
            tir::ExprKind::Unary(_, _) => unreachable!(),
            tir::ExprKind::Block(_) => unreachable!(),
            tir::ExprKind::ItemRef(_) => unreachable!(),
            tir::ExprKind::Tuple(_) => unreachable!(),
            tir::ExprKind::Lambda(_) => unreachable!(),
            tir::ExprKind::Call(_, _) => unreachable!(),
            tir::ExprKind::Match(_, _) => unreachable!(),
            tir::ExprKind::Assign(_, _) => unreachable!(),
            tir::ExprKind::Ret(_) => unreachable!(),
        }
    }
}

use crate::ast;
use crate::mir::build::*;
use crate::mir::*;
use crate::set;
use crate::span::Span;
use crate::ty::Ty;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn as_tmp(&mut self, mut block: BlockId, expr: &tir::Expr<'tcx>) -> BlockAnd<Lvalue<'tcx>> {
        let info = self.span_info(expr.span);
        let lvalue = self.alloc_tmp(info, expr.ty).into();
        match expr.kind {
            tir::ExprKind::Bin(op, l, r) => set!(block = self.write_expr(block, lvalue, expr)),
            tir::ExprKind::Const(_) => unreachable!(),
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
        };
        block.and(lvalue)
    }
}

use crate::ast;
use crate::mir::build::*;
use crate::mir::*;
use crate::set;
use crate::span::Span;
use crate::ty::Ty;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn as_tmp(&mut self, mut block: BlockId, expr: &tir::Expr<'tcx>) -> BlockAnd<VarId> {
        let info = self.span_info(expr.span);
        let var = self.alloc_tmp(info, expr.ty);
        // include a pattern below if some expressions require special treatment
        match expr.kind {
            _ => {
                set!(block = self.write_expr(block, var.into(), expr));
                block.and(var)
            }
        }
    }
}

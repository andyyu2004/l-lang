use super::*;
use crate::mir::*;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn build_closure(
        &mut self,
        block: BlockId,
        expr: &tir::Expr<'tcx>,
        body: &'tcx tir::Body<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        let body = build_fn(&self.ctx, body);
        block.and(Rvalue::Closure(expr.ty, body))
    }
}

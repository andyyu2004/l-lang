use super::*;
use crate::mir::*;
use build::ENTRY_BLOCK;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn build_closure(
        &mut self,
        block: BlockId,
        closure: &tir::Expr<'tcx>,
        upvars: &[tir::Expr<'tcx>],
        body: &'tcx tir::Body<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        let info = self.span_info(closure.span);
        let upvars =
            upvars.iter().map(|upvar| set!(block = self.as_operand(block, upvar))).collect_vec();
        todo!();
    }
}

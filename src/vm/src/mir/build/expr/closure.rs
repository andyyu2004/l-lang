use super::*;
use crate::mir::*;
use build::ENTRY_BLOCK;
use rustc_hash::FxHashMap;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn build_closure(
        &mut self,
        mut block: BlockId,
        closure: &tir::Expr<'tcx>,
        upvars: &[tir::Expr<'tcx>],
        body: &'tcx tir::Body<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        let info = self.span_info(closure.span);
        let upvars: FxHashMap<ir::Id, _> = upvars
            .iter()
            .map(|upvar| (upvar.id, set!(block = self.as_operand(block, upvar))))
            .collect();
        todo!()
    }
}

use super::*;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn build_closure(
        &mut self,
        mut block: BlockId,
        closure: &tir::Expr<'tcx>,
        upvars: &[tir::Expr<'tcx>],
        body: &tir::Body<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        let info = self.span_info(closure.span);
        let upvars =
            upvars.iter().map(|upvar| set!(block = self.as_operand(block, upvar))).collect_vec();
        todo!()
    }
}

use super::*;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn build_closure(
        &mut self,
        mut block: BlockId,
        closure: &tir::Expr<'tcx>,
        upvars: &[tir::Expr<'tcx>],
        _body: &tir::Body<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        let _info = self.span_info(closure.span);
        let _upvars =
            upvars.iter().map(|upvar| set!(block = self.as_operand(block, upvar))).collect_vec();
        todo!()
    }
}

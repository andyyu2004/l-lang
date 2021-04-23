use super::*;

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
    crate fn build_closure(
        &mut self,
        mut block: BlockId,
        closure: &tir::Expr<'tcx>,
        upvars: &[tir::Expr<'tcx>],
        _body: &tir::Body<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        // let _info = self.span_info(closure.span);
        // TODO
        // let upvars =
        //     upvars.iter().map(|upvar| set!(block = self.as_operand(block, upvar))).collect_vec();
        // we need this to not crash during tests so we have a random impl currently
        block.and(Rvalue::Closure { ty: closure.ty, upvars: vec![] })
    }
}

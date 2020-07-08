use super::InferCtx;
use std::ops::Deref;

crate struct FnCtx<'a, 'tcx> {
    infcx: &'a InferCtx<'a, 'tcx>,
    // locals: Env<Symbol, Ty<'tcx>>,
}

impl<'a, 'tcx> Deref for FnCtx<'a, 'tcx> {
    type Target = InferCtx<'a, 'tcx>;
    fn deref(&self) -> &Self::Target {
        &self.infcx
    }
}

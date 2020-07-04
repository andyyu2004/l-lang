use std::marker::PhantomData;

crate struct TyCtx<'tcx> {
    phantom: &'tcx PhantomData<()>,
}

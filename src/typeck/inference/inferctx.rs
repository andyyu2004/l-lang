use std::marker::PhantomData;

crate struct InferCtx<'cx, 'tcx> {
    cx: &'cx PhantomData<()>,
    tcx: &'tcx PhantomData<()>,
}

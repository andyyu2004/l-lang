use std::marker::PhantomData;

crate type Ty<'tcx> = &'tcx TyS<'tcx>;

crate struct TyS<'tcx> {
    kind: TyKind<'tcx>,
}

crate enum TyKind<'tcx> {
    Phantom(&'tcx PhantomData<()>),
}

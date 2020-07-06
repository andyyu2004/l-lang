crate struct Item<'tcx> {
    marker: std::marker::PhantomData<&'tcx ()>,
}

crate enum ItemKind {}

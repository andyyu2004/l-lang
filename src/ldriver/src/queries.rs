use ir::DefId;
use lcore::queries::{Queries, QueryProviders};
use lcore::ty::{self, Ty, TyCtx};

fn type_of<'tcx>(_tcx: TyCtx<'tcx>, _def_id: DefId) -> Ty<'tcx> {
    todo!()
}

fn generics_of<'tcx>(_tcx: TyCtx<'tcx>, _def_id: DefId) -> ty::Generics<'tcx> {
    todo!()
}

fn construct_query_providers() -> QueryProviders {
    QueryProviders { type_of, generics_of }
}

crate fn queries<'tcx>() -> Queries<'tcx> {
    Queries::new(construct_query_providers())
}

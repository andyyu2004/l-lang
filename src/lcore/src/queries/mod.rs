use std::cell::RefCell;

use crate::ty::{Generics, Ty, TyCtx};
use ir::DefId;
use rustc_hash::FxHashMap;

pub struct Queries<'tcx> {
    storage: QueryStorage<'tcx>,
    providers: QueryProviders,
}

impl<'tcx> TyCtx<'tcx> {
    // pub fn type_of(self, def_id: DefId) -> Ty<'tcx> {
    //     self.storage
    //         .type_of
    //         .borrow_mut()
    //         .entry(def_id)
    //         .or_insert_with(|| (self.providers.type_of)(self, def_id))
    // }

    // pub fn generics_of(self, def_id: DefId) -> Ty<'tcx> {
    //     self.storage
    //         .generics_of
    //         .borrow_mut()
    //         .entry(def_id)
    //         .or_insert_with(|| (self.providers.generics_of)(self, def_id))
    // }
}

impl<'tcx> Queries<'tcx> {
    pub fn new(providers: QueryProviders) -> Self {
        Self { providers, storage: Default::default() }
    }
}

#[derive(Default)]
pub struct QueryStorage<'tcx> {
    type_of: RefCell<FxHashMap<DefId, Ty<'tcx>>>,
    generics_of: RefCell<FxHashMap<DefId, Ty<'tcx>>>,
}

pub struct QueryProviders {
    pub type_of: for<'tcx> fn(TyCtx<'tcx>, DefId) -> Ty<'tcx>,
    pub generics_of: for<'tcx> fn(TyCtx<'tcx>, DefId) -> Generics<'tcx>,
}

// queries! {
//     type_of: for<'tcx> fn (TyCtx<'tcx>, DefId) -> Ty<'tcx>,
//     generics_of: for<'tcx> fn (TyCtx<'tcx>, DefId) -> Generics<'tcx>
// }

// #[macro_export]
// macro_rules! queries {
//     ($($name:ident:$ty:ty),*) => {
//         pub struct Queries {
//             pub $($name: $ty),*
//         }
//     };
// }

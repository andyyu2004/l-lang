use std::cell::RefCell;

use crate::ty::{Generics, Ty, TyCtx};
use ir::DefId;
use rustc_hash::FxHashMap;

pub struct QueryCtx<'tcx> {
    storage: QueryStorage<'tcx>,
    providers: Queries,
}

impl<'tcx> TyCtx<'tcx> {
    pub fn type_of(self, def_id: DefId) -> Ty<'tcx> {
        self.storage
            .type_of
            .borrow_mut()
            .entry(def_id)
            .or_insert_with(|| (self.providers.type_of)(self, def_id))
    }

    pub fn generics_of(self, def_id: DefId) -> &'tcx Generics<'tcx> {
        self.storage
            .generics_of
            .borrow_mut()
            .entry(def_id)
            .or_insert_with(|| (self.providers.generics_of)(self, def_id))
    }
}

impl<'tcx> QueryCtx<'tcx> {
    pub fn new(providers: Queries) -> Self {
        Self { providers, storage: Default::default() }
    }
}

#[derive(Default)]
pub struct QueryStorage<'tcx> {
    type_of: RefCell<FxHashMap<DefId, Ty<'tcx>>>,
    generics_of: RefCell<FxHashMap<DefId, &'tcx Generics<'tcx>>>,
}

/// query providers
pub struct Queries {
    pub type_of: for<'tcx> fn(TyCtx<'tcx>, DefId) -> Ty<'tcx>,
    pub generics_of: for<'tcx> fn(TyCtx<'tcx>, DefId) -> &'tcx Generics<'tcx>,
}

impl Queries {
    pub fn assert_is_fully_populated(&self) {
        assert_ne!(self.type_of as *const (), std::ptr::null());
        assert_ne!(self.generics_of as *const (), std::ptr::null());
    }
}

fn mk_null_fn<I, R>() -> for<'tcx> fn(TyCtx<'tcx>, I) -> R {
    unsafe { std::mem::transmute::<*const (), for<'tcx> fn(TyCtx<'tcx>, I) -> R>(std::ptr::null()) }
}

/// these default functions obviously need to be overwritten by providers
impl Default for Queries {
    fn default() -> Self {
        Self { type_of: mk_null_fn(), generics_of: mk_null_fn() }
    }
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

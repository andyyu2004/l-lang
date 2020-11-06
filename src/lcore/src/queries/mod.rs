use std::cell::RefCell;

use crate::ty::{AdtTy, Generics, Ty, TyCtx};
use ir::DefId;
use rustc_hash::FxHashMap;

pub struct QueryCtx<'tcx> {
    storage: QueryStorage<'tcx>,
    queries: Queries,
}

impl<'tcx> TyCtx<'tcx> {
    pub fn type_of(self, def_id: DefId) -> Ty<'tcx> {
        // written in this way to allow for recursive queries without `BorrowMut` errors
        if let Some(ty) = self.storage.types_cache.borrow().get(&def_id) {
            return ty;
        }
        let ty = (self.queries.type_of)(self, def_id);
        self.storage.types_cache.borrow_mut().insert(def_id, ty);
        // we must add a validation check for certain item types
        // as some constraints cannot be checked during `type_of`
        // this includes recursive adts
        // so we must call this after adding the type to the cache
        self.validate_item_ty(def_id);
        ty
    }

    pub fn validate_item_ty(self, def_id: DefId) {
        (self.queries.validate_item_ty)(self, def_id)
    }

    pub fn adt_ty(self, def_id: DefId) -> &'tcx AdtTy<'tcx> {
        if let Some(ty) = self.storage.adts_cache.borrow().get(&def_id) {
            return ty;
        }
        let adt = (self.queries.adt_ty)(self, def_id);
        self.storage.adts_cache.borrow_mut().insert(def_id, adt);
        adt
    }

    pub fn generics_of(self, def_id: DefId) -> &'tcx Generics<'tcx> {
        self.storage
            .generics_cache
            .borrow_mut()
            .entry(def_id)
            .or_insert_with(|| (self.queries.generics_of)(self, def_id))
    }

    pub fn inherent_impls(self) -> FxHashMap<DefId, Vec<DefId>> {
        self.storage
            .all_inherent_impls
            .borrow_mut()
            .get_or_insert_with(|| (self.queries.inherent_impls)(self))
            .clone()
    }

    pub fn inherent_impls_of(self, def_id: DefId) -> Vec<DefId> {
        self.inherent_impls().get(&def_id).cloned().unwrap_or_else(Vec::new)
    }
}

impl<'tcx> QueryCtx<'tcx> {
    pub fn new(providers: Queries) -> Self {
        Self { queries: providers, storage: Default::default() }
    }
}

#[derive(Default)]
pub struct QueryStorage<'tcx> {
    types_cache: RefCell<FxHashMap<DefId, Ty<'tcx>>>,
    adts_cache: RefCell<FxHashMap<DefId, &'tcx AdtTy<'tcx>>>,
    generics_cache: RefCell<FxHashMap<DefId, &'tcx Generics<'tcx>>>,
    all_inherent_impls: RefCell<Option<FxHashMap<DefId, Vec<DefId>>>>,
}

/// functions pointers to the functions that compute the query
pub struct Queries {
    pub type_of: for<'tcx> fn(TyCtx<'tcx>, DefId) -> Ty<'tcx>,
    pub adt_ty: for<'tcx> fn(TyCtx<'tcx>, DefId) -> &'tcx AdtTy<'tcx>,
    pub validate_item_ty: for<'tcx> fn(TyCtx<'tcx>, DefId),
    pub generics_of: for<'tcx> fn(TyCtx<'tcx>, DefId) -> &'tcx Generics<'tcx>,
    pub inherent_impls: for<'tcx> fn(TyCtx<'tcx>) -> FxHashMap<DefId, Vec<DefId>>,
}

fn mk_nullary_null_fn<R>() -> for<'tcx> fn(TyCtx<'tcx>) -> R {
    unsafe { std::mem::transmute::<*const (), for<'tcx> fn(TyCtx<'tcx>) -> R>(std::ptr::null()) }
}

fn mk_unary_null_fn<I, R>() -> for<'tcx> fn(TyCtx<'tcx>, I) -> R {
    unsafe { std::mem::transmute::<*const (), for<'tcx> fn(TyCtx<'tcx>, I) -> R>(std::ptr::null()) }
}

/// these default functions obviously need to be overwritten by providers
impl Default for Queries {
    fn default() -> Self {
        Self {
            type_of: mk_unary_null_fn(),
            adt_ty: mk_unary_null_fn(),
            generics_of: mk_unary_null_fn(),
            validate_item_ty: mk_unary_null_fn(),
            inherent_impls: mk_nullary_null_fn(),
        }
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

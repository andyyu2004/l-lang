use crate::mir::Mir;
use crate::ty::{AdtTy, Generics, InherentImpls, Instance, Ty, TyCtx};
use error::LResult;
use ir::DefId;
use rustc_hash::FxHashMap;
use std::cell::RefCell;

macro_rules! define_queries {
    (tcx: $tcx:tt, inputs: { $(([$name:ident] [$K:ty] [$R:ty]))* }) => {
        pub struct Queries {
            $(pub $name: for<$tcx> fn(TyCtx<$tcx>, $K) -> $R,)*
        }

        impl Default for Queries {
            fn default() -> Self {
                $(fn $name<$tcx>(_: TyCtx<$tcx>, key: $K) -> $R {
                    panic!("`tcx.{}({:?})` unprovided", stringify!($name), key);
                })*
                Self { $($name),* }
            }
        }
    };
}

macro_rules! define_query_caches {
    (tcx: $tcx:tt, inputs: {$(([$name:ident] [$K:ty] [$R:ty]))*}) => {
        #[derive(Default)]
        pub struct QueryCache<$tcx> {
            $($name: RefCell<FxHashMap<$K, $R>>),*
        }
    };
}

macro_rules! define_query_context {
    (tcx: $tcx:tt, inputs: {$(([$name:ident] [$K:ty] [$R:ty]))*}) => {

        define_queries!(tcx: $tcx, inputs: { $(([$name] [$K] [$R]))* });
        define_query_caches!(tcx: $tcx, inputs: { $(([$name] [$K] [$R]))* });

        pub struct QueryCtx<'tcx> {
            cache: QueryCache<'tcx>,
            queries: Queries,
        }

        impl<'tcx> QueryCtx<'tcx> {
            pub fn new(queries: Queries) -> Self {
                Self { queries, cache: Default::default()
            }
        }
}

        impl<'tcx> TyCtx<'tcx> {
            $(pub fn $name(self, key: $K) -> $R {
                // we must have a early return otherwise we will run into
                // refcell `BorrowMut` errors
                if let Some(value) = self.cache.$name.borrow().get(&key) {
                    return value.clone()
                }


                let computed = (self.queries.$name)(self, key);
                self.cache.$name.borrow_mut().insert(key, computed.clone());
                computed
            })*
        }
    };
}

define_query_context! {
    tcx: 'tcx,
    inputs: {
        // typecheck
        ([type_of] [DefId] [Ty<'tcx>])
        ([adt_ty] [DefId] [&'tcx AdtTy<'tcx>])
        ([generics_of] [DefId] [&'tcx Generics<'tcx>])
        ([validate_item_ty] [DefId] [()])
        ([inherent_impls] [()] [InherentImpls])
        ([inherent_impls_of] [DefId] [Vec<DefId>])

        // mir
        ([mir_of] [DefId] [LResult<&'tcx Mir<'tcx>>])
        ([instance_mir] [Instance<'tcx>] [LResult<&'tcx Mir<'tcx>>])
    }
}

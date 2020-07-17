use crate::arena::{DropArena, DroplessArena, TypedArena};
use crate::ir::Definitions;
use crate::ty::{List, SubstRef, Ty, TyKind, TyS};
use std::{alloc::Layout, marker::PhantomData};

/// collective arena which contains all main arenas
#[derive(Default)]
crate struct Arena<'tcx> {
    dropless: DroplessArena,
    drop: DropArena,
    tys: TypedArena<TyS<'tcx>>,
    tir: DroplessArena,
    substs: TypedArena<SubstRef<'tcx>>,
    // phantom data for each type that may be allocated in drop
    def_marker: PhantomData<Definitions>,
}

impl<'tcx> Arena<'tcx> {
    pub fn alloc<T>(&self, t: T) -> &T {
        if !std::mem::needs_drop::<T>() {
            self.dropless.alloc(t)
        } else {
            unsafe { self.drop.alloc(t) }
        }
    }

    pub fn alloc_raw(&self, layout: Layout) -> *mut u8 {
        self.dropless.alloc_raw(layout)
    }

    pub fn alloc_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
        let ty_structure = TyS { kind };
        self.tys.alloc(ty_structure)
    }

    pub fn alloc_tir<T>(&self, tir: T) -> &T {
        self.tir.alloc(tir)
    }

    pub fn alloc_tir_iter<I, T>(&self, iter: I) -> &[T]
    where
        I: IntoIterator<Item = T>,
    {
        self.tir.alloc_from_iter(iter)
    }
}

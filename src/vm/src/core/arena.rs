use crate::arena::{DropArena, DroplessArena, TypedArena};
use crate::ir::{self, Definitions};
use crate::ty::{Const, List, SubstsRef, Ty, TyFlag, TyKind, TyS};
use crate::typeck::TypeckOutputs;
use std::alloc::Layout;
use std::marker::PhantomData;

/// collective arena which contains all main arenas
#[derive(Default)]
pub struct Arena<'tcx> {
    pub ir: DroplessArena,
    drop: DropArena,
    dropless: DroplessArena,
    tys: TypedArena<TyS<'tcx>>,
    consts: TypedArena<Const<'tcx>>,
    substs: TypedArena<SubstsRef<'tcx>>,
    tir: DroplessArena,
    // phantom data for each type that may be allocated in drop
    def_marker: PhantomData<Definitions>,
    prog_marker: PhantomData<ir::Prog<'tcx>>,
    typeck_outputs_marker: PhantomData<TypeckOutputs<'tcx>>,
}

impl<'tcx> Arena<'tcx> {
    pub fn alloc<T>(&self, t: T) -> &T {
        if !std::mem::needs_drop::<T>() {
            self.dropless.alloc(t)
        } else {
            unsafe { self.drop.alloc(t) }
        }
    }

    pub fn alloc_iter<I, T>(&self, t: I) -> &[T]
    where
        I: IntoIterator<Item = T>,
    {
        self.dropless.alloc_from_iter(t)
    }

    pub fn alloc_raw(&self, layout: Layout) -> *mut u8 {
        self.dropless.alloc_raw(layout)
    }

    pub fn alloc_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
        let flags = kind.ty_flags();
        let ty_structure = TyS { kind, flags };
        self.tys.alloc(ty_structure)
    }

    pub fn alloc_const(&'tcx self, c: Const<'tcx>) -> &'tcx Const<'tcx> {
        self.consts.alloc(c)
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

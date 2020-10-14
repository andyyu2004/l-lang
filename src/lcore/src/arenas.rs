// use crate::ir::{self, Definitions};
// use crate::ty::{Const, List, SubstsRef, Ty, TyFlag, TyKind, TyS};
// use crate::typeck::TypeckTables;
use crate::ty::*;
use arena::{DropArena, DroplessArena, TypedArena};
use ir::Definitions;
use std::alloc::Layout;
use std::marker::PhantomData;

/// collective arena which contains all main arenas
#[derive(Default)]
pub struct CoreArenas<'tcx> {
    drop: DropArena,
    dropless: DroplessArena,
    tys: TypedArena<TyS<'tcx>>,
    consts: TypedArena<Const<'tcx>>,
    // phantom data for each type that may be allocated in drop
    def_marker: PhantomData<Definitions<'tcx>>,
    prog_marker: PhantomData<ir::IR<'tcx>>,
    // typeck_tables_marker: PhantomData<TypeckTables<'tcx>>,
    tmp: PhantomData<&'tcx ()>,
}

impl<'tcx> CoreArenas<'tcx> {
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
}

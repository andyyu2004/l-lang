use crate::core::Arena;
use crate::ty::{List, SubstsRef, Ty, TyKind, TyS};
use rustc_hash::FxHashMap;
use std::{borrow::Borrow, cell::RefCell};

pub struct CtxInterners<'tcx> {
    pub arena: &'tcx Arena<'tcx>,
    /// map from tykind to the allocated ty ptr
    types: RefCell<FxHashMap<TyKind<'tcx>, Ty<'tcx>>>,
    /// map from a slice of tys to a allocated SubstsRef
    /// this uses the fact the default slice equality is implemented by length + element wise comparison
    substs: RefCell<FxHashMap<&'tcx [Ty<'tcx>], SubstsRef<'tcx>>>,
}

impl<'tcx> CtxInterners<'tcx> {
    pub fn new(arena: &'tcx Arena<'tcx>) -> Self {
        Self { arena, types: Default::default(), substs: Default::default() }
    }

    pub(crate) fn intern_ty(&self, kind: TyKind<'tcx>) -> Ty<'tcx> {
        let mut types = self.types.borrow_mut();
        match types.get(&kind) {
            Some(ty) => *ty,
            None => {
                let ty = self.arena.alloc_ty(kind.clone());
                types.insert(kind, ty);
                ty
            }
        }
    }

    pub(crate) fn intern_substs(&self, slice: &[Ty<'tcx>]) -> SubstsRef<'tcx> {
        let mut substs = self.substs.borrow_mut();
        match substs.get(slice) {
            Some(&substs_ref) => substs_ref,
            None => {
                let substs_ref = List::from_arena(self.arena, slice);
                substs.insert(&substs_ref, substs_ref);
                substs_ref
            }
        }
    }
}

use crate::core::Arena;
use crate::ty::{Ty, TyKind, TyS};
use rustc_hash::FxHashMap;
use std::{borrow::Borrow, cell::RefCell};

crate struct CtxInterners<'tcx> {
    pub arena: &'tcx Arena<'tcx>,
    /// map from tykind to the allocated ty ptr
    types: RefCell<FxHashMap<TyKind<'tcx>, Ty<'tcx>>>,
}

impl<'tcx> Borrow<TyKind<'tcx>> for TyS<'tcx> {
    fn borrow(&self) -> &TyKind<'tcx> {
        todo!()
    }
}

impl<'tcx> CtxInterners<'tcx> {
    pub fn new(arena: &'tcx Arena<'tcx>) -> Self {
        Self { arena, types: Default::default() }
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
}

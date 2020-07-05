use crate::{
    shared::Arena, ty::TyKind, ty::{Ty, TyS}
};
use rustc_hash::FxHashMap;
use std::borrow::Borrow;

crate struct CtxInterners<'tcx> {
    arena: &'tcx Arena<'tcx>,
    /// map from tykind to the allocated ty ptr
    types: FxHashMap<TyKind<'tcx>, Ty<'tcx>>,
}

impl<'tcx> Borrow<TyKind<'tcx>> for TyS<'tcx> {
    fn borrow(&self) -> &TyKind<'tcx> {
        todo!()
    }
}

impl<'tcx> CtxInterners<'tcx> {
    pub fn new(arena: &'tcx Arena<'tcx>) -> Self {
        Self {
            arena,
            types: Default::default(),
        }
    }

    pub(crate) fn intern_ty(&mut self, kind: TyKind<'tcx>) -> Ty<'tcx> {
        match self.types.get(&kind) {
            Some(ty) => *ty,
            None => {
                let ty = self.arena.alloc_ty(kind.clone());
                self.types.insert(kind, ty);
                ty
            }
        }
    }

    pub fn intern_tir<T>(&self, tir: T) -> &T {
        self.arena.alloc_tir(tir)
    }
}

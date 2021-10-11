use crate::ty::*;
use crate::Arena;
use rustc_hash::{FxHashMap, FxHasher};
use std::cell::RefCell;
use std::collections::hash_map::RawEntryMut;
use std::hash::{Hash, Hasher};

pub struct CtxInterners<'tcx> {
    /// general arena
    pub arena: &'tcx Arena<'tcx>,
    /// map from tykind to the allocated ty ptr
    types: RefCell<FxHashMap<TyKind<'tcx>, Ty<'tcx>>>,
    /// map from a slice of tys to a allocated SubstsRef
    /// this uses the fact the default slice equality is implemented by
    /// length + element wise comparison
    substs: RefCell<FxHashMap<&'tcx [Ty<'tcx>], SubstsRef<'tcx>>>,
    lvalue_projs: RefCell<FxHashMap<&'tcx [Projection<'tcx>], &'tcx List<Projection<'tcx>>>>,
    // need a hashmap for the raw_entry api
    consts: RefCell<FxHashMap<&'tcx Const<'tcx>, ()>>,
}

impl<'tcx> CtxInterners<'tcx> {
    pub fn new(arena: &'tcx Arena<'tcx>) -> Self {
        Self {
            arena,
            types: Default::default(),
            substs: Default::default(),
            consts: Default::default(),
            lvalue_projs: Default::default(),
        }
    }

    pub fn intern_ty(&self, kind: TyKind<'tcx>) -> Ty<'tcx> {
        let mut types = self.types.borrow_mut();
        match types.get(&kind) {
            Some(&ty) => ty,
            None => {
                let flags = kind.ty_flags();
                let ty = self.arena.alloc(Type { kind, flags });
                types.insert(kind, ty);
                ty
            }
        }
    }

    pub fn intern_lvalue_projections(
        &self,
        projs: &[Projection<'tcx>],
    ) -> &'tcx List<Projection<'tcx>> {
        let mut projections = self.lvalue_projs.borrow_mut();
        match projections.get(projs) {
            Some(&projs) => projs,
            None => {
                let projs = List::from_arena(self.arena, projs);
                projections.insert(&projs, projs);
                projs
            }
        }
    }

    pub fn intern_const(&self, c: Const<'tcx>) -> &'tcx Const<'tcx> {
        // this method avoids needing to clone `c`
        let mut consts = self.consts.borrow_mut();
        let hash = fx_hash(&c);
        let entry = consts.raw_entry_mut().from_key_hashed_nocheck(hash, &c);
        match entry {
            RawEntryMut::Occupied(e) => e.key(),
            RawEntryMut::Vacant(e) => {
                let c = self.arena.alloc(c);
                e.insert_hashed_nocheck(hash, c, ());
                c
            }
        }
    }

    pub fn intern_substs(&self, slice: &[Ty<'tcx>]) -> SubstsRef<'tcx> {
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

#[inline]
fn fx_hash<K: Hash + ?Sized>(val: &K) -> u64 {
    let mut state = FxHasher::default();
    val.hash(&mut state);
    state.finish()
}

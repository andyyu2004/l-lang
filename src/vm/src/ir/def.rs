use crate::ir::{self, DefId, ParamIdx, VariantIdx};
use indexed_vec::{Idx, IndexVec};
use rustc_hash::FxHashMap;
use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Res<Id = ir::Id> {
    PrimTy(ir::PrimTy),
    Def(DefId, DefKind<Id>),
    Local(Id),
    Err,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CtorKind {
    /// Option::Some(x)
    Tuple,
    /// Option::Some { x }
    Struct,
    /// Option::None
    Unit,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DefKind<Id = ir::Id> {
    Fn,
    Enum,
    Struct,
    /// constructor of enum variant
    /// `DefId` is the parent of the adt itself
    Ctor(CtorKind, VariantIdx, Id),
    /// contains the index of the `TyParam` in its scope
    /// impl<T, U> Foo<T, U> {
    ///     fn bar<V> () { .. }
    /// }
    /// (T, U, V) would have indices (0,1,2) respectively
    TyParam(ParamIdx),
}

impl<Id> DefKind<Id> {
    pub fn map_id<R>(self, f: impl FnOnce(Id) -> R) -> DefKind<R> {
        match self {
            DefKind::Ctor(kind, idx, id) => DefKind::Ctor(kind, idx, f(id)),
            DefKind::Fn => DefKind::Fn,
            DefKind::TyParam(idx) => DefKind::TyParam(idx),
            DefKind::Enum => DefKind::Enum,
            DefKind::Struct => DefKind::Struct,
        }
    }
}

impl<Id> Res<Id> {
    pub fn map_id<R>(self, f: impl FnOnce(Id) -> R) -> Res<R> {
        match self {
            Res::PrimTy(ty) => Res::PrimTy(ty),
            Res::Local(id) => Res::Local(f(id)),
            Res::Def(def_id, def_kind) => Res::Def(def_id, def_kind.map_id(f)),
            Res::Err => Res::Err,
        }
    }
}

#[derive(Default, Debug)]
pub struct Definitions {
    /// just use a counter for DefIds for now
    def_id_counter: Cell<usize>,
    id_to_def_id: FxHashMap<ir::Id, DefId>,
    def_id_to_hir_id: IndexVec<DefId, Option<ir::Id>>,
}

impl Definitions {
    pub fn alloc_def_id(&self) -> DefId {
        let def_id = self.def_id_counter.get();
        self.def_id_counter.set(1 + def_id);
        DefId::new(def_id)
    }
}

/// namespaces for types and values
#[derive(Debug, Clone, Copy)]
pub enum NS {
    Type,
    Value,
}

/// a `T` for each namespace
#[derive(Default, Debug)]
pub struct PerNS<T> {
    pub value: T,
    pub ty: T,
}

impl<T> std::ops::Index<NS> for PerNS<T> {
    type Output = T;

    fn index(&self, ns: NS) -> &Self::Output {
        match ns {
            NS::Value => &self.value,
            NS::Type => &self.ty,
        }
    }
}

impl<T> std::ops::IndexMut<NS> for PerNS<T> {
    fn index_mut(&mut self, ns: NS) -> &mut Self::Output {
        match ns {
            NS::Value => &mut self.value,
            NS::Type => &mut self.ty,
        }
    }
}

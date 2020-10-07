use super::{DefId, ParamIdx};
use crate as ir;
use index::{Idx, IndexVec};
use rustc_hash::FxHashMap;
use std::cell::Cell;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Res<Id = ir::Id> {
    PrimTy(ir::PrimTy),
    Def(DefId, DefKind),
    Local(Id),
    SelfTy,
    Err,
}

pub trait HasDefKind {
    fn def_kind(&self) -> DefKind;
}

impl HasDefKind for ast::AssocItemKind {
    fn def_kind(&self) -> DefKind {
        match self {
            Self::Fn(..) => DefKind::AssocFn,
        }
    }
}

impl HasDefKind for ast::ItemKind {
    fn def_kind(&self) -> DefKind {
        match self {
            ast::ItemKind::Fn(..) => DefKind::Fn,
            ast::ItemKind::Enum(..) => DefKind::Enum,
            ast::ItemKind::Struct(..) => DefKind::Struct,
            ast::ItemKind::Impl { .. } => DefKind::Impl,
            ast::ItemKind::Extern(..) => DefKind::Extern,
        }
    }
}

impl<Id> Display for Res<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let description = match *self {
            Res::Def(_, kind) => return write!(f, "{}", kind),
            Res::PrimTy(..) => "builtin type",
            Res::Local(..) => "local variable",
            Res::SelfTy => "self",
            Res::Err => "unresolved item",
        };
        write!(f, "{}", description)
    }
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

impl CtorKind {
    pub fn is_function(self) -> bool {
        match self {
            CtorKind::Tuple => true,
            CtorKind::Struct | CtorKind::Unit => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DefKind {
    Fn,
    AssocFn,
    Enum,
    Struct,
    Impl,
    /// extern block
    Extern,
    /// constructor of enum variant
    /// `DefId` is the parent of the adt itself
    Ctor(CtorKind),
    /// contains the index of the `TyParam` in its scope
    /// impl<T, U> Foo<T, U> {
    ///     fn bar<V> () { .. }
    /// }
    /// (T, U, V) would have indices (0,1,2) respectively
    TyParam(ParamIdx),
}

impl Display for DefKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DefKind::Fn => write!(f, "function"),
            DefKind::AssocFn => write!(f, "associated function"),
            DefKind::Impl => write!(f, "impl"),
            DefKind::Enum => write!(f, "enum"),
            DefKind::Struct => write!(f, "struct"),
            DefKind::Ctor(ctor) => match ctor {
                CtorKind::Tuple => write!(f, "tuple constructor"),
                CtorKind::Struct => write!(f, "struct constructor"),
                CtorKind::Unit => write!(f, "unit constructor"),
            },
            DefKind::TyParam(_) => write!(f, "type parameter"),
            DefKind::Extern => write!(f, "extern block"),
        }
    }
}

impl<Id> Res<Id> {
    pub fn map_id<R>(self, f: impl FnOnce(Id) -> R) -> Res<R> {
        match self {
            Res::PrimTy(ty) => Res::PrimTy(ty),
            Res::Local(id) => Res::Local(f(id)),
            Res::Def(def_id, def_kind) => Res::Def(def_id, def_kind),
            Res::SelfTy => Res::SelfTy,
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

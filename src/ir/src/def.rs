use super::{DefId, ParamIdx};
use crate as ir;
use ast::NodeId;
use index::Idx;
use rustc_hash::FxHashMap;
use std::cell::Cell;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Res<Id = ir::Id> {
    Err,
    Def(DefId, DefKind),
    // type namespace
    PrimTy(ir::PrimTy),
    SelfTy { impl_def: DefId },
    // value namespace
    Local(Id),
    SelfVal { impl_def: DefId },
}

impl<Id> Res<Id> {
    pub fn expect_def(self) -> (DefId, DefKind) {
        match self {
            Res::Def(def_id, def_kind) => (def_id, def_kind),
            _ => panic!(),
        }
    }
}

/// partial resolution
/// resolves things that can be resolved early such as modules and constructor paths
/// foo::bar or Option::Some
/// defers resolution of associated items (such as associated functions and associated types)
/// for typechecking phase
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PartialRes {
    /// the resolution for the resolved portion of the path
    pub resolved: Res<NodeId>,
    /// the number of unresolved segments
    pub unresolved: usize,
}

impl PartialRes {
    pub fn new(resolved: Res<NodeId>, unresolved: usize) -> Self {
        Self { resolved, unresolved }
    }

    pub fn resolved(resolved: Res<NodeId>) -> Self {
        Self::new(resolved, 0)
    }
}

pub trait HasDefKind {
    fn def_kind(&self) -> DefKind;
}

impl HasDefKind for ast::ForeignItemKind {
    fn def_kind(&self) -> DefKind {
        match self {
            // does a foreign function require its own defkind?
            ast::ForeignItemKind::Fn(_, _) => DefKind::Fn,
        }
    }
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
            Res::SelfTy { .. } => "Self type",
            Res::Err => "unresolved item",
            Res::SelfVal { .. } => "Self value",
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
            Res::SelfTy { impl_def } => Res::SelfTy { impl_def },
            Res::SelfVal { impl_def } => Res::SelfVal { impl_def },
            Res::Err => Res::Err,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DefNode<'ir> {
    Item(ir::Item<'ir>),
    ImplItem(ir::ImplItem<'ir>),
    ForeignItem(ir::ForeignItem<'ir>),
    /// the node is considered a ctor iff it is a tuple variant
    Ctor(ir::Variant<'ir>),
    Variant(ir::Variant<'ir>),
    TyParam(&'ir ir::TyParam<'ir>),
}

impl<'ir> Into<DefNode<'ir>> for ir::Item<'ir> {
    fn into(self) -> DefNode<'ir> {
        DefNode::Item(self)
    }
}

impl<'ir> Into<DefNode<'ir>> for &'ir ir::TyParam<'ir> {
    fn into(self) -> DefNode<'ir> {
        DefNode::TyParam(self)
    }
}

impl<'ir> Into<DefNode<'ir>> for ir::ImplItem<'ir> {
    fn into(self) -> DefNode<'ir> {
        DefNode::ImplItem(self)
    }
}

impl<'ir> Into<DefNode<'ir>> for ir::ForeignItem<'ir> {
    fn into(self) -> DefNode<'ir> {
        DefNode::ForeignItem(self)
    }
}

impl<'ir> Into<DefNode<'ir>> for ir::Variant<'ir> {
    fn into(self) -> DefNode<'ir> {
        if self.kind.is_tuple() { DefNode::Ctor(self) } else { DefNode::Variant(self) }
    }
}

#[derive(Default, Debug)]
pub struct Definitions<'a> {
    /// just use a counter for DefIds for now
    def_id_counter: Cell<usize>,
    // id_to_def_id: FxHashMap<ir::Id, DefId>,
    // def_id_to_ir_id: IndexVec<DefId, Option<ir::Id>>,
    def_map: FxHashMap<DefId, DefNode<'a>>,
}

impl<'a> Definitions<'a> {
    /// adds def mapping from `def_id` to `node`
    pub fn def_node(&mut self, def_id: DefId, node: DefNode<'a>) {
        assert!(self.def_map.insert(def_id, node).is_none());
    }

    pub fn get_def_node(&self, def_id: DefId) -> DefNode<'a> {
        self.def_map[&def_id]
    }

    pub fn alloc_def_id(&self) -> DefId {
        let def_id = self.def_id_counter.get();
        self.def_id_counter.set(1 + def_id);
        DefId::new(def_id)
    }
}

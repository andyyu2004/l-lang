#![feature(bindings_after_at)]
#![feature(decl_macro)]

#[cfg(test)]
mod tests;

mod def_visitor;
mod late;
mod module;
mod pat;
mod resolution_error;
mod scope;

#[macro_use]
extern crate log;

use late::LateResolver;
use module::Module;
use pat::PatternResolutionCtx;
use resolution_error::ResolutionError;
use scope::{Scope, Scopes};

use arena::TypedArena;
use ast::{Ident, NodeId, Prog};
use error::MultiSpan;
use index::IndexVec;
use ir::{DefId, DefKind, Definitions, ModuleId, ParamIdx, PrimTy, Res, ROOT_MODULE};
use rustc_hash::FxHashMap;
use session::Session;
use span::{kw, sym, Symbol};
use std::error::Error;
use std::ops::{Deref, Index, IndexMut};

#[derive(Default)]
pub struct ResolverArenas<'a> {
    pub modules: TypedArena<Module<'a>>,
}

pub struct Resolver<'a> {
    arenas: &'a ResolverArenas<'a>,
    primitive_types: PrimitiveTypes,
    modules: IndexVec<ModuleId, &'a Module<'a>>,
    defs: Definitions<'a>,
    /// (usize, usize) is (min, max) number of type parameters expected
    /// (as some may be default parameters)
    generic_arg_counts: FxHashMap<DefId, usize>,
    sess: &'a Session,
    /// map of resolved `NodeId`s to its resolution
    res_map: FxHashMap<NodeId, Res<NodeId>>,
    node_id_to_def_id: FxHashMap<NodeId, DefId>,
    ty_param_id_to_idx: FxHashMap<NodeId, ParamIdx>,
}

/// stuff that is useful later in `TyCtx` that the resolver computes
#[derive(Debug, Default)]
pub struct Resolutions<'a> {
    pub defs: Definitions<'a>,
    pub generic_arg_counts: FxHashMap<DefId, usize>,
}

impl<'a> Resolver<'a> {
    pub fn new(sess: &'a Session, arenas: &'a ResolverArenas<'a>) -> Self {
        Self {
            sess,
            arenas,
            modules: IndexVec::from_elem_n(arenas.modules.alloc(Module::root()), 1),
            generic_arg_counts: Default::default(),
            res_map: Default::default(),
            defs: Default::default(),
            node_id_to_def_id: Default::default(),
            primitive_types: Default::default(),
            ty_param_id_to_idx: Default::default(),
        }
    }

    /// top level function to run the resolver on the given prog
    pub fn resolve(&mut self, prog: &Prog) {
        self.resolve_items(prog);
        self.late_resolve(prog);
    }

    pub fn complete(self) -> Resolutions<'a> {
        let Resolver { defs, generic_arg_counts, .. } = self;
        Resolutions { defs, generic_arg_counts }
    }

    pub fn find_module(&mut self, par: ModuleId, ident: Ident) -> Option<ModuleId> {
        let par = self.modules[par];
        par.submodules.borrow().get(&ident).copied()
    }

    pub fn root_module(&mut self) -> &Module<'a> {
        self.modules[ROOT_MODULE]
    }

    pub fn new_module(&mut self, par: ModuleId, name: Ident) -> ModuleId {
        let module = self.arenas.modules.alloc(Module::default());
        let id = self.modules.push(module);
        if self.modules[par].submodules.borrow_mut().insert(name, id).is_some() {
            self.emit_error(name.span, ResolutionError::DuplicateModuleDefinition(name));
        };
        id
    }

    /// allocates a `DefId` for some given `NodeId`
    pub fn def(&mut self, _name: Ident, node_id: NodeId) -> DefId {
        let def_id = self.defs.alloc_def_id();
        assert!(self.node_id_to_def_id.insert(node_id, def_id).is_none());
        def_id
    }

    pub fn def_node(&mut self, def_id: DefId, node: ir::DefNode<'a>) {
        self.defs.def_node(def_id, node)
    }

    pub fn emit_error(&self, span: impl Into<MultiSpan>, err: impl Error) -> Res<NodeId> {
        self.sess.emit_error(span, err);
        Res::Err
    }

    pub fn def_ty_param(&mut self, id: NodeId, idx: ParamIdx) -> Res<NodeId> {
        self.ty_param_id_to_idx.insert(id, idx);
        Res::Def(self.def_id(id), DefKind::TyParam(idx))
    }

    pub fn idx_of_ty_param(&mut self, id: NodeId) -> ParamIdx {
        self.ty_param_id_to_idx.get(&id).copied().unwrap()
    }

    pub fn def_item(
        &mut self,
        module: ModuleId,
        name: Ident,
        node_id: NodeId,
        def_kind: DefKind,
    ) -> DefId {
        let def_id = self.def(name, node_id);
        if name.symbol == kw::Empty {
            // nameless items such as extern blocks and impls don't need to be added to the
            // module's items as they cannot be referenced by identifier
            return def_id;
        }
        if self.modules[module]
            .items
            .borrow_mut()
            .insert(name, Res::Def(def_id, def_kind))
            .is_some()
        {
            self.emit_error(name.span, ResolutionError::DuplicateDefinition(name));
        };
        def_id
    }

    pub fn resolve_item(&self, module: ModuleId, ident: Ident) -> Option<Res<NodeId>> {
        self.modules[module].items.borrow().get(&ident).copied()
    }

    /// node_id -> def_id
    pub fn def_id(&self, node_id: NodeId) -> DefId {
        self.node_id_to_def_id.get(&node_id).copied().unwrap_or_else(|| {
            panic!("unresolved def_id for node `{:?}` (check def_visitor implementation)", node_id)
        })
    }

    pub fn get_res(&self, id: NodeId) -> Res<NodeId> {
        *self.res_map.get(&id).unwrap_or_else(|| panic!("unresolved node `{:?}`", id))
    }

    /// writes the resolution for a given `NodeId` into the map
    fn resolve_node(&mut self, node_id: NodeId, res: Res<NodeId>) {
        info!("resolving node {:?} to {:?}", node_id, res);
        if let Some(prev_res) = self.res_map.insert(node_id, res) {
            // not sure why its resolving some stuff twice, but make sure its the same
            assert_eq!(res, prev_res);
        }
    }

    pub fn record_generic_arg_count(&mut self, def_id: DefId, argc: usize) {
        assert!(self.generic_arg_counts.insert(def_id, argc).is_none())
    }
}

#[derive(Debug)]
pub struct PrimitiveTypes {
    types: FxHashMap<Symbol, PrimTy>,
}

impl Default for PrimitiveTypes {
    fn default() -> Self {
        let mut types = FxHashMap::default();
        types.insert(sym::bool, PrimTy::Bool);
        types.insert(sym::float, PrimTy::Float);
        types.insert(sym::int, PrimTy::Int);
        types.insert(sym::char, PrimTy::Char);
        Self { types }
    }
}

impl Deref for PrimitiveTypes {
    type Target = FxHashMap<Symbol, PrimTy>;

    fn deref(&self) -> &Self::Target {
        &self.types
    }
}

impl<T> Index<NS> for PerNS<T> {
    type Output = T;

    fn index(&self, ns: NS) -> &Self::Output {
        match ns {
            NS::Value => &self.value,
            NS::Type => &self.ty,
        }
    }
}

impl<T> IndexMut<NS> for PerNS<T> {
    fn index_mut(&mut self, ns: NS) -> &mut Self::Output {
        match ns {
            NS::Value => &mut self.value,
            NS::Type => &mut self.ty,
        }
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

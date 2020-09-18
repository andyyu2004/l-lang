use super::{Module, ModuleTree};
use crate::arena::TypedArena;
use crate::ast::{Ident, NodeId, Prog};
use crate::driver::Session;
use crate::error::ResolutionError;
use crate::ir::{DefId, DefKind, Definitions, ModuleId, ParamIdx, PrimTy, Res, ROOT_MODULE};
use crate::lexer::{symbol, Symbol};
use crate::span::Span;
use indexed_vec::IndexVec;
use rustc_hash::FxHashMap;
use std::error::Error;
use std::marker::PhantomData;

#[derive(Default)]
pub struct ResolverArenas<'a> {
    modules: TypedArena<Module<'a>>,
}

pub struct Resolver<'a> {
    arenas: &'a ResolverArenas<'a>,
    root: ModuleTree<'a>,
    modules: IndexVec<ModuleId, &'a Module<'a>>,
    defs: Definitions,
    sess: &'a Session,
    /// map of resolved `NodeId`s to its resolution
    res_map: FxHashMap<NodeId, Res<NodeId>>,
    node_id_to_def_id: FxHashMap<NodeId, DefId>,
    ty_param_id_to_idx: FxHashMap<NodeId, ParamIdx>,
    pub(super) primitive_types: PrimitiveTypes,
}

#[derive(Debug)]
pub struct ResolverOutputs {
    pub defs: Definitions,
}

impl<'a> Resolver<'a> {
    pub fn new(sess: &'a Session, arenas: &'a ResolverArenas<'a>) -> Self {
        Self {
            sess,
            arenas,
            modules: IndexVec::from_elem_n(arenas.modules.alloc(Module::default()), 1),
            root: Default::default(),
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
        self.late_resolve_prog(prog);
    }

    pub fn complete(self) -> ResolverOutputs {
        let Resolver { defs, .. } = self;
        ResolverOutputs { defs }
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
    pub fn def(&mut self, name: Ident, node_id: NodeId) -> DefId {
        let def_id = self.defs.alloc_def_id();
        self.node_id_to_def_id.insert(node_id, def_id);
        def_id
    }

    pub fn emit_error(&self, span: Span, err: impl Error) -> Res<NodeId> {
        self.sess.emit_error(span, err);
        Res::Err
    }

    pub fn def_ty_param(&mut self, id: NodeId, idx: ParamIdx) -> Res<NodeId> {
        self.ty_param_id_to_idx.insert(id, idx);
        Res::Def(self.def_id(id), DefKind::TyParam(idx))
    }

    pub fn idx_of_ty_param(&mut self, id: NodeId) -> ParamIdx {
        *self.ty_param_id_to_idx.get(&id).unwrap()
    }

    pub fn def_item(
        &mut self,
        module: ModuleId,
        name: Ident,
        node_id: NodeId,
        def_kind: DefKind,
    ) -> DefId {
        let def_id = self.def(name, node_id);
        if self.modules[module]
            .items
            .borrow_mut()
            .insert(name, Res::Def(def_id, def_kind))
            .is_some()
        {
            self.emit_error(name.span, ResolutionError::DuplicateDefinition(def_kind, name));
        };
        def_id
    }

    pub fn resolve_item(&self, module: ModuleId, ident: Ident) -> Option<Res<NodeId>> {
        self.modules[module].items.borrow().get(&ident).copied()
    }

    /// node_id -> def_id
    pub fn def_id(&self, node_id: NodeId) -> DefId {
        self.node_id_to_def_id[&node_id]
    }

    pub fn get_res(&self, id: NodeId) -> Res<NodeId> {
        *self.res_map.get(&id).unwrap_or_else(|| panic!("unresolved node `{:?}`", id))
    }

    /// writes the resolution for a given `NodeId` into the map
    pub(super) fn resolve_node(&mut self, node_id: NodeId, res: Res<NodeId>) {
        self.res_map.insert(node_id, res);
    }
}

#[derive(Debug, Deref)]
pub struct PrimitiveTypes {
    types: FxHashMap<Symbol, PrimTy>,
}

impl Default for PrimitiveTypes {
    fn default() -> Self {
        let mut types = FxHashMap::default();
        types.insert(symbol::BOOL, PrimTy::Bool);
        types.insert(symbol::FLOAT, PrimTy::Float);
        types.insert(symbol::INT, PrimTy::Int);
        types.insert(symbol::CHAR, PrimTy::Char);
        Self { types }
    }
}

use crate::ast::{Ident, NodeId, Prog};
use crate::ir::{DefId, DefKind, Definitions, ParamIdx, PrimTy, Res};
use crate::lexer::{symbol, Symbol};
use indexed_vec::IndexVec;
use rustc_hash::FxHashMap;
use std::marker::PhantomData;

pub struct Resolver {
    defs: Definitions,
    /// map of resolved `NodeId`s to its resolution
    res_map: FxHashMap<NodeId, Res<NodeId>>,
    items: FxHashMap<Ident, Res<NodeId>>,
    node_id_to_def_id: FxHashMap<NodeId, DefId>,
    ty_param_id_to_idx: FxHashMap<NodeId, ParamIdx>,
    pub(super) primitive_types: PrimitiveTypes,
}

#[derive(Debug)]
pub struct ResolverOutputs {
    pub defs: Definitions,
}

impl Resolver {
    /// construct a resolver and run resolution
    pub fn resolve(prog: &Prog) -> Self {
        let mut resolver = Self {
            items: Default::default(),
            res_map: Default::default(),
            defs: Default::default(),
            node_id_to_def_id: Default::default(),
            primitive_types: Default::default(),
            ty_param_id_to_idx: Default::default(),
        };
        resolver.resolve_prog(prog);
        resolver
    }

    pub fn complete(self) -> ResolverOutputs {
        let Resolver { defs, .. } = self;
        ResolverOutputs { defs }
    }

    pub fn def_item(&mut self, name: Ident, node_id: NodeId, def_kind: DefKind) -> DefId {
        let def_id = self.def(name, node_id);
        self.items.insert(name, Res::Def(def_id, def_kind));
        def_id
    }

    /// allocates a `DefId` for some given `NodeId`
    pub fn def(&mut self, name: Ident, node_id: NodeId) -> DefId {
        let def_id = self.defs.alloc_def_id();
        self.node_id_to_def_id.insert(node_id, def_id);
        def_id
    }

    pub fn def_ty_param(&mut self, id: NodeId, idx: ParamIdx) -> Res<NodeId> {
        self.ty_param_id_to_idx.insert(id, idx);
        Res::Def(self.def_id(id), DefKind::TyParam(idx))
    }

    pub fn idx_of_ty_param(&mut self, id: NodeId) -> ParamIdx {
        *self.ty_param_id_to_idx.get(&id).unwrap()
    }

    pub fn resolve_item(&mut self, ident: Ident) -> Option<Res<NodeId>> {
        self.items.get(&ident).copied()
    }

    /// top level function to run the resolver on the given prog
    pub fn resolve_prog(&mut self, prog: &Prog) {
        self.resolve_items(prog);
        self.late_resolve_prog(prog);
    }

    /// node_id -> def_id
    pub fn def_id(&self, node_id: NodeId) -> DefId {
        self.node_id_to_def_id[&node_id]
    }

    pub fn get_res(&self, id: NodeId) -> Res<NodeId> {
        *self.res_map.get(&id).unwrap()
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

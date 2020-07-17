use crate::ast::{NodeId, Prog};
use crate::ir::{DefId, Definitions, PrimTy, Res};
use crate::lexer::{symbol, Symbol};
use indexed_vec::IndexVec;
use rustc_hash::FxHashMap;
use std::marker::PhantomData;

crate struct Resolver {
    defs: Definitions,
    /// map of resolved `NodeId`s to its resolution
    res_map: FxHashMap<NodeId, Res<NodeId>>,
    node_id_to_def_id: FxHashMap<NodeId, DefId>,
    pub(super) primitive_types: PrimitiveTypes,
}

#[derive(Debug)]
crate struct ResolverOutputs {
    pub defs: Definitions,
}

impl Resolver {
    /// construct a resolver and run resolution
    pub fn resolve(prog: &Prog) -> Self {
        let mut resolver = Self {
            res_map: Default::default(),
            defs: Default::default(),
            node_id_to_def_id: Default::default(),
            primitive_types: Default::default(),
        };
        resolver.resolve_prog(prog);
        resolver
    }

    pub fn complete(self) -> ResolverOutputs {
        let Resolver { defs, .. } = self;
        ResolverOutputs { defs }
    }

    pub fn create_def(&mut self, node_id: NodeId) -> DefId {
        let def_id = self.defs.alloc_def_id();
        self.node_id_to_def_id.insert(node_id, def_id);
        def_id
    }

    /// top level function to run the resolver on the given prog
    pub fn resolve_prog(&mut self, prog: &Prog) {
        self.late_resolve_prog(prog);
    }

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
crate struct PrimitiveTypes {
    types: FxHashMap<Symbol, PrimTy>,
}

impl Default for PrimitiveTypes {
    fn default() -> Self {
        let mut types = FxHashMap::default();
        types.insert(symbol::BOOL, PrimTy::Bool);
        types.insert(symbol::NUMBER, PrimTy::Num);
        types.insert(symbol::CHAR, PrimTy::Char);
        Self { types }
    }
}

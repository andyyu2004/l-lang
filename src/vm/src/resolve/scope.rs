use crate::ast::{Ident, NodeId};
use crate::ir::Res;
use rustc_hash::FxHashMap;

crate struct Scopes {
    scopes: Vec<Scope>,
}

crate struct Scope {
    bindings: FxHashMap<Ident, Res<NodeId>>,
}

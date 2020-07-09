use crate::ast::{NodeId, Prog};
use crate::ir::Res;
use rustc_hash::FxHashMap;

crate struct Resolver {
    res_map: FxHashMap<NodeId, Res<NodeId>>,
}

impl Resolver {
}

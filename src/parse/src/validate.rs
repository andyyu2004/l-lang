use crate::{NodeId, Visitor};
use rustc_hash::FxHashSet;

#[derive(Default)]
pub struct AstValidator {
    ids: FxHashSet<NodeId>,
}

impl<'ast> Visitor<'ast> for AstValidator {
    fn visit_id(&mut self, id: NodeId) {
        assert!(self.ids.insert(id))
    }
}

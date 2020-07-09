use crate::arena::DroplessArena as Arena;
use crate::ast::NodeId;
use crate::ir::{self, DefId, Id, LocalId};
use indexed_vec::{Idx, IndexVec};

crate struct LoweringCtx<'ir> {
    crate arena: &'ir Arena,
    node_id_to_id: IndexVec<NodeId, Option<ir::Id>>,
    item_stack: Vec<(DefId, usize)>,
}

impl<'ir> LoweringCtx<'ir> {
    pub fn new(arena: &'ir Arena) -> Self {
        Self { arena, item_stack: Default::default(), node_id_to_id: Default::default() }
    }

    pub(super) fn lower_node_id(&mut self, node_id: NodeId) -> ir::Id {
        self.lower_node_id_generic(node_id, |this| {
            let &mut (def_id, ref mut counter) = this.item_stack.last_mut().unwrap();
            let local_id = *counter;
            *counter += 1;
            Id { def_id, local_id: LocalId::new(local_id) }
        })
    }

    fn lower_node_id_generic(
        &mut self,
        node_id: NodeId,
        mk_id: impl FnOnce(&mut Self) -> ir::Id,
    ) -> ir::Id {
        let min_size = 1 + node_id.index();
        if min_size > self.node_id_to_id.len() {
            self.node_id_to_id.resize(min_size, None);
        }
        if let Some(existing_id) = self.node_id_to_id[node_id] {
            existing_id
        } else {
            let new_id = mk_id(self);
            self.node_id_to_id[node_id] = Some(new_id);
            new_id
        }
    }
}

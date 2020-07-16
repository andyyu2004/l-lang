use crate::arena::DroplessArena as Arena;
use crate::ast::*;
use crate::ir::{self, DefId, Id, LocalId, Res};
use crate::resolve::Resolver;
use indexed_vec::{Idx, IndexVec};
use std::collections::BTreeMap;

crate struct AstLoweringCtx<'a, 'ir> {
    pub(super) arena: &'ir Arena,
    pub(super) resolver: &'a mut Resolver,
    pub(super) node_id_to_id: IndexVec<NodeId, Option<ir::Id>>,
    pub(super) item_stack: Vec<(DefId, usize)>,
}

impl<'a, 'ir> AstLoweringCtx<'a, 'ir> {
    pub fn new(arena: &'ir Arena, resolver: &'a mut Resolver) -> Self {
        Self { arena, item_stack: Default::default(), resolver, node_id_to_id: Default::default() }
    }

    pub fn lower_prog(mut self, prog: &Prog) -> ir::Prog<'ir> {
        let items = prog
            .items
            .iter()
            .map(|item| self.lower_item(item))
            .map(|item| (item.id, item))
            .collect();
        ir::Prog { items }
    }

    pub(super) fn with_owner<T>(&mut self, owner: NodeId, f: impl FnOnce(&mut Self) -> T) -> T {
        let def_id = self.resolver.def_id(owner);
        self.item_stack.push((def_id, 0));
        let ret = f(self);
        let (popped_def_id, popped_counter) = self.item_stack.pop().unwrap();
        debug_assert_eq!(popped_def_id, def_id);
        ret
    }

    pub(super) fn lower_node_id(&mut self, node_id: NodeId) -> ir::Id {
        self.lower_node_id_generic(node_id, |this| {
            let &mut (def_id, ref mut counter) = this.item_stack.last_mut().unwrap();
            let local_id = *counter;
            *counter += 1;
            Id { def_id, local: LocalId::new(local_id) }
        })
    }

    pub(super) fn lower_body(&mut self, sig: &FnSig, expr: &Expr) -> &'ir ir::Body<'ir> {
        let params = self.lower_params(&sig.inputs);
        let sig = self.lower_fn_sig(sig);
        let expr = self.lower_expr(expr);
        self.arena.alloc(ir::Body { params, expr })
    }

    pub(super) fn lower_params(&mut self, params: &[Param]) -> &'ir [ir::Param<'ir>] {
        self.arena.alloc_from_iter(params.iter().map(|p| self.lower_param(p)))
    }

    pub(super) fn lower_param(&mut self, param: &Param) -> ir::Param<'ir> {
        let span = param.span;
        let id = self.lower_node_id(param.id);
        let pattern = self.lower_pattern(&param.pattern);
        ir::Param { span, id, pat: pattern }
    }

    pub(super) fn lower_res(&mut self, res: Res<NodeId>) -> Res {
        res.map_id(|id| self.lower_node_id(id))
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

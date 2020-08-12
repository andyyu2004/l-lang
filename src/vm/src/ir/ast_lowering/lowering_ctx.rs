use crate::arena::DroplessArena as Arena;
use crate::ast::*;
use crate::ir::{self, DefId, Id, LocalId, Res};
use crate::resolve::Resolver;
use crate::span::Span;
use indexed_vec::{Idx, IndexVec};
use rustc_hash::FxHashMap;
use std::{cell::Cell, collections::BTreeMap};

pub struct AstLoweringCtx<'a, 'ir> {
    pub(super) arena: &'ir Arena,
    pub(super) resolver: &'a mut Resolver,
    pub(super) node_id_to_id: FxHashMap<NodeId, ir::Id>,
    pub(super) item_stack: Vec<(DefId, usize)>,
    /// this counter counts backwards as to be sure not to not
    /// overlap with the ids that the parser assigned
    node_id_counter: Cell<usize>,
}

/// methods for constructing `ir` for desugaring purposes
impl<'a, 'ir> AstLoweringCtx<'a, 'ir> {
    pub(super) fn mk_expr(&mut self, span: Span, kind: ir::ExprKind<'ir>) -> &'ir ir::Expr<'ir> {
        self.arena.alloc(ir::Expr { id: self.new_id(), span, kind })
    }

    pub(super) fn mk_expr_bool(&mut self, span: Span, b: bool) -> &'ir ir::Expr<'ir> {
        self.mk_expr(span, ir::ExprKind::Lit(Lit::Bool(b)))
    }

    pub(super) fn mk_pat_bool(&mut self, span: Span, b: bool) -> &'ir ir::Pattern<'ir> {
        let expr = self.mk_expr_bool(span, b);
        self.mk_pat(span, ir::PatternKind::Lit(expr))
    }

    pub(super) fn mk_pat(
        &mut self,
        span: Span,
        kind: ir::PatternKind<'ir>,
    ) -> &'ir ir::Pattern<'ir> {
        self.arena.alloc(ir::Pattern { id: self.new_id(), span, kind })
    }

    pub(super) fn mk_empty_block_expr(&mut self, span: Span) -> &'ir ir::Expr<'ir> {
        let block = self.mk_empty_block(span);
        self.mk_expr(span, ir::ExprKind::Block(block))
    }

    pub(super) fn mk_empty_block(&mut self, span: Span) -> &'ir ir::Block<'ir> {
        self.arena.alloc(ir::Block { id: self.new_id(), span, stmts: &[], expr: None })
    }

    pub(super) fn mk_arm(
        &mut self,
        pat: &'ir ir::Pattern<'ir>,
        expr: &'ir ir::Expr<'ir>,
    ) -> ir::Arm<'ir> {
        ir::Arm { id: self.new_id(), span: pat.span.merge(expr.span), pat, guard: None, body: expr }
    }
}

impl<'a, 'ir> AstLoweringCtx<'a, 'ir> {
    pub fn new(arena: &'ir Arena, resolver: &'a mut Resolver) -> Self {
        Self {
            arena,
            item_stack: Default::default(),
            resolver,
            node_id_to_id: Default::default(),
            node_id_counter: Cell::new(u32::MAX as usize - 1),
        }
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

    fn mk_node_id(&self) -> NodeId {
        let c = self.node_id_counter.get();
        self.node_id_counter.set(c - 1);
        NodeId::new(c)
    }

    pub(super) fn new_id(&mut self) -> ir::Id {
        let node_id = self.mk_node_id();
        self.lower_node_id(node_id)
    }

    pub(super) fn lower_node_id(&mut self, node_id: NodeId) -> ir::Id {
        self.lower_node_id_generic(node_id, |this| {
            let &mut (def, ref mut counter) = this.item_stack.last_mut().unwrap();
            let local_id = *counter;
            *counter += 1;
            Id { def, local: LocalId::new(local_id) }
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
        match self.node_id_to_id.get(&node_id) {
            Some(&existing) => existing,
            None => {
                let new_id = mk_id(self);
                self.node_id_to_id.insert(node_id, new_id);
                new_id
            }
        }
    }
}

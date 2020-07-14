use super::AstLoweringCtx;
use crate::ast::{FnSig, Item, ItemKind, Param};
use crate::ir::{self, *};
use std::marker::PhantomData;

impl<'a, 'ir> AstLoweringCtx<'a, 'ir> {
    pub fn lower_item(&mut self, item: &Item) -> &'ir ir::Item<'ir> {
        self.with_owner(item.id, |lctx| {
            let &Item { span, id, vis, ref kind, ident } = item;
            let kind = match &kind {
                ItemKind::Fn(sig, generics, block) => {
                    // assume the function has a body for now
                    let block = lctx.lower_block(block.as_ref().unwrap());
                    let lowered_sig = lctx.lower_fn_sig(sig);
                    let generics = lctx.arena.alloc(Generics { pd: PhantomData, data: 0 });
                    let params = lctx
                        .arena
                        .alloc_from_iter(sig.inputs.iter().map(|param| lctx.lower_param(param)));
                    let body = ir::Body { params, expr: lctx.arena.alloc(Expr::from(block)) };
                    ir::ItemKind::Fn(lowered_sig, generics, lctx.arena.alloc(body))
                }
            };
            lctx.arena.alloc(ir::Item { span, id: lctx.lower_node_id(id), vis, ident, kind })
        })
    }

    fn lower_param(&mut self, param: &Param) -> ir::Param<'ir> {
        let span = param.span;
        let id = self.lower_node_id(param.id);
        let pattern = self.lower_pattern(&param.pattern);
        ir::Param { span, id, pat: pattern }
    }

    fn lower_fn_sig(&mut self, sig: &FnSig) -> &'ir ir::FnSig<'ir> {
        let inputs =
            self.arena.alloc_from_iter(sig.inputs.iter().map(|p| self.lower_ty_inner(&p.ty)));
        let output = sig.output.as_ref().map(|ty| self.lower_ty(ty));
        self.arena.alloc(ir::FnSig { inputs, output })
    }
}

use super::AstLoweringCtx;
use crate::ast::{FnSig, Item, ItemKind, Param};
use crate::ir::{self, *};
use std::marker::PhantomData;

impl<'a, 'ir> AstLoweringCtx<'a, 'ir> {
    pub fn lower_item(&mut self, item: &Item) -> &'ir ir::Item<'ir> {
        self.with_owner(item.id, |lctx| {
            let &Item { span, id, vis, ref kind, ident } = item;
            let kind = match &kind {
                ItemKind::Fn(sig, generics, expr) => {
                    // assume the function has a body for now
                    let lowered_sig = lctx.lower_fn_sig(sig);
                    let body = lctx.lower_body(sig, expr.as_ref().unwrap());
                    let lowered_sig = lctx.lower_fn_sig(sig);
                    let generics = lctx.arena.alloc(Generics { pd: PhantomData, data: 0 });
                    ir::ItemKind::Fn(lowered_sig, generics, body)
                }
            };
            lctx.arena.alloc(ir::Item { span, id: lctx.lower_node_id(id), vis, ident, kind })
        })
    }

    pub(super) fn lower_fn_sig(&mut self, sig: &FnSig) -> &'ir ir::FnSig<'ir> {
        let inputs =
            self.arena.alloc_from_iter(sig.inputs.iter().map(|p| self.lower_ty_inner(&p.ty)));
        let output = sig.output.as_ref().map(|ty| self.lower_ty(ty));
        self.arena.alloc(ir::FnSig { inputs, output })
    }
}

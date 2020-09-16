use super::AstLoweringCtx;
use crate::ast::*;
use crate::ir::{self, DefKind, Res};
use crate::lexer::Symbol;
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
                    let generics = lctx.lower_generics(generics);
                    ir::ItemKind::Fn(lowered_sig, generics, body)
                }
                ItemKind::Enum(generics, variants) => {
                    let generics = lctx.lower_generics(generics);
                    let variants = lctx
                        .arena
                        .alloc_from_iter(variants.iter().map(|v| lctx.lower_variant(item, v)));
                    ir::ItemKind::Enum(generics, variants)
                }
                ItemKind::Struct(generics, variant_kind) => {
                    let generics = lctx.lower_generics(generics);
                    let kind = lctx.lower_variant_kind(variant_kind);
                    ir::ItemKind::Struct(generics, kind)
                }
            };
            lctx.arena.alloc(ir::Item { span, id: lctx.lower_node_id(id), vis, ident, kind })
        })
    }

    fn lower_variant(&mut self, item: &Item, variant: &Variant) -> ir::Variant<'ir> {
        self.with_owner(variant.id, |lctx| ir::Variant {
            id: lctx.lower_node_id(variant.id),
            ident: variant.ident,
            span: variant.span,
            kind: lctx.lower_variant_kind(&variant.kind),
        })
    }

    fn lower_field_decl(&mut self, (i, field): (usize, &FieldDecl)) -> ir::FieldDecl<'ir> {
        let &FieldDecl { span, ident, vis, id, ref ty } = field;
        let ident = ident.unwrap_or_else(|| Ident { span: field.span, symbol: Symbol::intern(i) });
        ir::FieldDecl { span, ident, vis, id: self.lower_node_id(id), ty: self.lower_ty(ty) }
    }

    fn lower_variant_kind(&mut self, variant_kind: &VariantKind) -> ir::VariantKind<'ir> {
        match variant_kind {
            VariantKind::Tuple(fields) => ir::VariantKind::Tuple(
                self.arena
                    .alloc_from_iter(fields.iter().enumerate().map(|f| self.lower_field_decl(f))),
            ),
            VariantKind::Struct(fields) => ir::VariantKind::Struct(
                self.arena
                    .alloc_from_iter(fields.iter().enumerate().map(|f| self.lower_field_decl(f))),
            ),
            VariantKind::Unit => ir::VariantKind::Unit,
        }
    }

    fn lower_generics(&mut self, generics: &Generics) -> &'ir ir::Generics<'ir> {
        let &Generics { span, ref params } = generics;
        let params = self.arena.alloc_from_iter(params.iter().map(|p| self.lower_ty_param(p)));
        self.arena.alloc(ir::Generics { span, params })
    }

    fn lower_ty_param(&mut self, param: &TyParam) -> ir::TyParam<'ir> {
        // `TyParam`s have their own `DefId`
        self.with_owner(param.id, |lctx| {
            let &TyParam { span, id, ident, ref default } = param;
            ir::TyParam {
                span,
                id: lctx.lower_node_id(id),
                index: lctx.resolver.idx_of_ty_param(id),
                ident,
                default: default.as_ref().map(|ty| lctx.lower_ty(ty)),
            }
        })
    }

    pub(super) fn lower_fn_sig(&mut self, sig: &FnSig) -> &'ir ir::FnSig<'ir> {
        let inputs =
            self.arena.alloc_from_iter(sig.params.iter().map(|p| self.lower_ty_inner(&p.ty)));
        let output = sig.ret_ty.as_ref().map(|ty| self.lower_ty(ty));
        self.arena.alloc(ir::FnSig { inputs, output })
    }
}

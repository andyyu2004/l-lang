use super::AstLoweringCtx;
use lc_ast::*;
use lc_index::Idx;
use ir::{DefId, DefNode, VariantIdx};
use lc_span::sym;
use lc_span::Symbol;

impl<'a, 'ir> AstLoweringCtx<'a, 'ir> {
    pub fn lower_items(&mut self, item: &[P<Item>]) {
        item.iter().for_each(|item| self.lower_item(item));
    }

    pub fn lower_item(&mut self, item: &Item) {
        self.with_def_id(item.id, |lctx| {
            let &Item { span, id, vis, ref kind, ident } = item;
            let id = lctx.lower_node_id(id);
            let kind = match &kind {
                ItemKind::Fn(sig, generics, expr) => {
                    if ident.symbol == sym::main {
                        lctx.entry_id = Some(id.def);
                    }
                    if expr.is_none() {
                        lctx.sess.emit_error(item.span, AstError::FunctionWithoutBody);
                        return;
                    }
                    let lowered_sig = lctx.lower_fn_sig(sig);
                    let generics = lctx.lower_generics(generics);
                    let body = lctx.lower_body(sig, expr.as_ref().unwrap());
                    ir::ItemKind::Fn(lowered_sig, generics, body)
                }
                ItemKind::Enum(generics, variants) => {
                    let generics = lctx.lower_generics(generics);
                    let variants = lctx.lower_variants(variants);
                    ir::ItemKind::Enum(generics, variants)
                }
                ItemKind::Struct(generics, variant_kind) => {
                    let generics = lctx.lower_generics(generics);
                    let kind = lctx.lower_variant_kind(variant_kind);
                    ir::ItemKind::Struct(generics, kind)
                }
                ItemKind::Extern(abi, items) => lctx.lower_foreign_mod(*abi, items),
                ItemKind::TypeAlias(generics, ty) => {
                    let generics = lctx.lower_generics(generics);
                    let ty = lctx.lower_ty(ty);
                    ir::ItemKind::TypeAlias(generics, ty)
                }
                ItemKind::Use(path) => ir::ItemKind::Use(lctx.lower_path(path)),
                ItemKind::Mod(module) => ir::ItemKind::Mod(lctx.lower_module(module)),
                ItemKind::Macro(_) => todo!(),
                ItemKind::Impl { generics, trait_path, self_ty, items } =>
                    lctx.lower_impl(generics, trait_path.as_ref(), self_ty, items),
                ItemKind::Trait { generics, items } => ir::ItemKind::Trait {
                    generics: lctx.lower_generics(generics),
                    trait_item_refs: lctx
                        .arena
                        .alloc_from_iter(items.iter().map(|item| lctx.lower_trait_item_ref(item))),
                },
            };
            let item = lctx.alloc(ir::Item { span, id, vis, ident, kind });
            lctx.mk_def_node(id.def, item);
            lctx.items.insert(id.def, item);
        });
    }

    pub(crate) fn lower_module(&mut self, module: &Module) -> ir::Mod<'ir> {
        let items = self.arena.alloc_from_iter(module.items.iter().map(|item| {
            self.lower_item(item);
            self.lower_node_id(item.id).def
        }));
        ir::Mod { span: module.span, items }
    }

    /// inserts DefId -> DefNode mapping into the `DefMap`
    /// returns the same T for convenience
    pub(crate) fn mk_def_node<T>(&mut self, def_id: DefId, node: T)
    where
        T: Into<DefNode<'ir>>,
    {
        self.resolver.mk_def_node(def_id, node.into());
    }

    fn lower_foreign_mod(&mut self, abi: Abi, items: &[P<ForeignItem>]) -> ir::ItemKind<'ir> {
        let foreign_items = self.lower_foreign_items(abi, items);
        foreign_items.iter().for_each(|item| self.mk_def_node(item.id.def, item));
        ir::ItemKind::Extern(abi, foreign_items)
    }

    fn lower_foreign_items(
        &mut self,
        abi: Abi,
        items: &[P<ForeignItem>],
    ) -> &'ir [ir::ForeignItem<'ir>] {
        self.arena.alloc_from_iter(items.iter().map(|item| self.lower_foreign_item(abi, item)))
    }

    fn lower_foreign_item(&mut self, abi: Abi, item: &ForeignItem) -> ir::ForeignItem<'ir> {
        let &ForeignItem { span, id, vis, ident, ref kind } = item;
        self.with_def_id(id, |lctx| {
            let id = lctx.lower_node_id(id);
            let kind = match kind {
                ForeignItemKind::Fn(sig, generics) =>
                    ir::ForeignItemKind::Fn(lctx.lower_fn_sig(sig), lctx.lower_generics(generics)),
            };
            ir::ForeignItem { id, abi, ident, span, vis, kind }
        })
    }

    fn lower_impl(
        &mut self,
        generics: &Generics,
        path: Option<&Path>,
        self_ty: &Ty,
        impl_items: &[Box<AssocItem>],
    ) -> ir::ItemKind<'ir> {
        let generics = self.lower_generics(generics);
        let trait_path = path.map(|path| self.lower_path(path));
        let self_ty = self.lower_ty(self_ty);
        let impl_item_refs = self
            .arena
            .alloc_from_iter(impl_items.iter().map(|item| self.lower_impl_item_ref(item)));
        ir::ItemKind::Impl { generics, trait_path, self_ty, impl_item_refs }
    }

    fn lower_trait_item_ref(&mut self, trait_item: &TraitItem) -> ir::TraitItemRef {
        self.with_def_id(trait_item.id, |lctx| {
            let item = lctx.lower_trait_item(trait_item);
            let id = ir::TraitItemId(item.id.def);
            lctx.trait_items.insert(id, item);
            ir::TraitItemRef { id }
        })
    }

    fn lower_trait_item(&mut self, trait_item: &TraitItem) -> &'ir ir::TraitItem<'ir> {
        let &TraitItem { id, ident, vis, span, ref kind } = trait_item;
        let id = self.lower_node_id(id);
        let (generics, kind) = match kind {
            AssocItemKind::Fn(sig, generics, body) => (
                generics,
                ir::TraitItemKind::Fn(
                    self.lower_fn_sig(sig),
                    body.as_ref().map(|body| self.lower_body(sig, body)),
                ),
            ),
        };
        let generics = self.lower_generics(generics);
        let trait_item = self.alloc(ir::TraitItem { id, ident, vis, span, generics, kind });
        self.mk_def_node(id.def, trait_item);
        trait_item
    }

    fn lower_impl_item_ref(&mut self, impl_item: &AssocItem) -> ir::ImplItemRef {
        self.with_def_id(impl_item.id, |lctx| {
            let item = lctx.lower_impl_item(impl_item);
            let id = ir::ImplItemId(item.id.def);
            lctx.impl_items.insert(id, item);
            ir::ImplItemRef { id }
        })
    }

    fn lower_impl_item(&mut self, impl_item: &AssocItem) -> &'ir ir::ImplItem<'ir> {
        let &AssocItem { span, id, vis, ident, ref kind } = impl_item;
        let id = self.lower_node_id(id);
        let (generics, kind) = match kind {
            AssocItemKind::Fn(sig, generics, body) => {
                let generics = self.lower_generics(generics);
                let body = self.lower_body(sig, body.as_deref().unwrap());
                let sig = self.lower_fn_sig(sig);
                (generics, ir::ImplItemKind::Fn(sig, body))
            }
        };

        let impl_def_id = self.parent_def_id(id);
        let impl_item =
            self.alloc(ir::ImplItem { id, impl_def_id, ident, span, vis, generics, kind });
        self.mk_def_node(id.def, impl_item);
        impl_item
    }

    fn lower_variants(&mut self, variants: &[Variant]) -> &'ir [ir::Variant<'ir>] {
        let variants = self
            .arena
            .alloc_from_iter(variants.iter().enumerate().map(|(i, v)| self.lower_variant(i, v)));
        variants.iter().for_each(|variant| self.mk_def_node(variant.id.def, variant));
        variants
    }

    fn lower_variant(&mut self, idx: usize, variant: &Variant) -> ir::Variant<'ir> {
        let adt_def_id = self.curr_owner();
        self.with_def_id(variant.id, |lctx| {
            let id = lctx.lower_node_id(variant.id);
            let kind = lctx.lower_variant_kind(&variant.kind);
            ir::Variant {
                id,
                kind,
                adt_def_id,
                ident: variant.ident,
                span: variant.span,
                idx: VariantIdx::new(idx),
            }
        })
    }

    fn lower_field_decls(&mut self, fields: &[FieldDecl]) -> &'ir [ir::FieldDecl<'ir>] {
        let fields =
            self.arena.alloc_from_iter(fields.iter().enumerate().map(|f| self.lower_field_decl(f)));
        fields.iter().for_each(|field| self.mk_def_node(field.id.def, field));
        fields
    }

    fn lower_field_decl(&mut self, (i, field): (usize, &FieldDecl)) -> ir::FieldDecl<'ir> {
        self.with_def_id(field.id, |lctx| {
            let &FieldDecl { span, ident, vis, id, ref ty } = field;
            // if it is a tuple struct/variant, the field will just be named after its index
            let ident =
                ident.unwrap_or_else(|| Ident::new(field.span, Symbol::intern(&i.to_string())));
            ir::FieldDecl { span, ident, vis, id: lctx.lower_node_id(id), ty: lctx.lower_ty(ty) }
        })
    }

    fn lower_variant_kind(&mut self, variant_kind: &VariantKind) -> ir::VariantKind<'ir> {
        match variant_kind {
            VariantKind::Tuple(fields) => ir::VariantKind::Tuple(self.lower_field_decls(fields)),
            VariantKind::Struct(fields) => ir::VariantKind::Struct(self.lower_field_decls(fields)),
            VariantKind::Unit => ir::VariantKind::Unit,
        }
    }

    pub(crate) fn lower_fn_sig(&mut self, sig: &FnSig) -> &'ir ir::FnSig<'ir> {
        let inputs =
            self.arena.alloc_from_iter(sig.params.iter().map(|p| self.lower_ty_inner(&p.ty)));
        let output = sig.ret_ty.as_ref().map(|ty| self.lower_ty(ty));
        self.arena.alloc(ir::FnSig { inputs, output })
    }
}

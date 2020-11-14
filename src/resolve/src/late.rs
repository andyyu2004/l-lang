use crate::*;
use ast::*;
use index::Idx;
use ir::{ParamIdx, Res};
use span::kw;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct LateResolver<'a, 'r, 'ast> {
    crate resolver: &'a mut Resolver<'r>,
    crate scopes: PerNS<Scopes<Res<NodeId>>>,
    crate current_module: Vec<ModuleId>,
    _pd: &'ast PhantomData<()>,
}

impl<'a, 'r, 'ast> LateResolver<'a, 'r, 'ast> {
    pub fn new(resolver: &'a mut Resolver<'r>) -> Self {
        Self {
            resolver,
            scopes: Default::default(),
            current_module: vec![ROOT_MODULE],
            _pd: &PhantomData,
        }
    }

    crate fn with_module<R>(&mut self, name: Ident, f: impl FnOnce(&mut Self) -> R) -> R {
        let module_id = self.resolve_module(name).unwrap();
        self.with_module_id(module_id, f)
    }

    crate fn with_module_id<R>(&mut self, module: ModuleId, f: impl FnOnce(&mut Self) -> R) -> R {
        self.current_module.push(module);
        let ret = f(self);
        self.current_module.pop();
        ret
    }

    crate fn with_val_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.scopes[NS::Value].push(Scope::default());
        let ret = f(self);
        self.scopes[NS::Value].pop();
        ret
    }

    pub fn with_ty_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.scopes[NS::Type].push(Scope::default());
        let ret = f(self);
        self.scopes[NS::Type].pop();
        ret
    }

    fn with_generics<R>(&mut self, generics: &Generics, f: impl FnOnce(&mut Self) -> R) -> R {
        self.with_ty_scope(|this| {
            for param in &generics.params {
                let index = this.scopes[NS::Type].def_ty_param();
                let res = this.resolver.def_ty_param(param.id, ParamIdx::new(index));
                this.scopes[NS::Type].def(param.ident, res)
            }
            f(this)
        })
    }

    crate fn def_val(&mut self, ident: Ident, res: Res<NodeId>) {
        self.scopes[NS::Value].def(ident, res);
    }

    fn resolve_pattern(&mut self, pat: &'ast Pattern) {
        PatternResolutionCtx::new(self).resolve_pattern(pat);
    }

    crate fn curr_module(&self) -> ModuleId {
        self.current_module.last().copied().unwrap()
    }

    /// searches for an item with name = `ident` in the current module
    crate fn try_resolve_item(&self, ident: Ident) -> Option<Res<NodeId>> {
        self.resolver.resolve_item(self.curr_module(), ident)
    }

    /// search for a local variable in scope otherwise look for a resolution to an item
    crate fn resolve_ident(&mut self, ident: Ident) -> Option<Res<NodeId>> {
        match self.scopes[NS::Value].lookup(ident) {
            Some(&res) => Some(res),
            None => self.try_resolve_item(ident),
        }
    }

    fn resolve_item(&mut self, item: &'ast Item) {
        match &item.kind {
            ItemKind::Fn(_, g, _) | ItemKind::TypeAlias(g, _) =>
                self.with_generics(g, |r| ast::walk_item(r, item)),
            ItemKind::Enum(g, _) | ItemKind::Struct(g, _) => self.resolve_adt(g, item),
            ItemKind::Impl { generics, trait_path, self_ty, items } =>
                self.resolve_impl(item, generics, trait_path.as_ref(), self_ty, items),
            ItemKind::Extern(..) => ast::walk_item(self, item),
            ItemKind::Mod(module) =>
                self.with_module(item.ident, |this| ast::walk_module(this, module)),
            ItemKind::Use(..) => {}
        }
    }

    fn resolve_foreign_item(&mut self, item: &'ast ForeignItem) {
        match &item.kind {
            ForeignItemKind::Fn(sig, generics) =>
                self.with_generics(generics, |this| this.visit_fn(sig, None)),
        }
    }

    fn with_self<R>(&mut self, impl_id: NodeId, f: impl FnOnce(&mut Self) -> R) -> R {
        self.with_self_ty(impl_id, |this| this.with_self_val(impl_id, f))
    }

    fn with_self_val<R>(&mut self, impl_id: NodeId, f: impl FnOnce(&mut Self) -> R) -> R {
        self.with_val_scope(|this| {
            let impl_def = this.resolver.def_id(impl_id);
            this.def_val(Ident::unspanned(kw::USelf), Res::SelfVal { impl_def });
            f(this)
        })
    }

    fn with_self_ty<R>(&mut self, impl_id: NodeId, f: impl FnOnce(&mut Self) -> R) -> R {
        self.with_ty_scope(|this| {
            let impl_def = this.resolver.def_id(impl_id);
            this.scopes[NS::Type].def(Ident::unspanned(kw::USelf), Res::SelfTy { impl_def });
            f(this)
        })
    }

    fn resolve_impl(
        &mut self,
        item: &'ast Item,
        generics: &'ast Generics,
        trait_path: Option<&'ast Path>,
        self_ty: &'ast Ty,
        assoc_items: &'ast [Box<AssocItem>],
    ) {
        self.with_generics(generics, |this| {
            if let Some(path) = trait_path {
                this.resolve_path(path, NS::Type);
            }
            this.visit_ty(self_ty);
            if trait_path.is_some() {
                todo!()
            }
            this.with_self(item.id, |this| {
                for item in assoc_items {
                    this.resolve_assoc_item(item);
                }
            })
        })
    }

    fn resolve_assoc_item(&mut self, item: &'ast AssocItem) {
        match &item.kind {
            // TODO add the impls generics to the assoc fns generics
            AssocItemKind::Fn(_, generics, _) =>
                self.with_generics(generics, |this| ast::walk_assoc_item(this, item)),
        }
    }

    fn resolve_adt(&mut self, generics: &'ast Generics, item: &'ast Item) {
        self.with_generics(generics, |this| ast::walk_item(this, item))
    }
}

impl<'a, 'ast> ast::Visitor<'ast> for LateResolver<'a, '_, 'ast> {
    fn visit_block(&mut self, block: &'ast Block) {
        self.with_val_scope(|this| ast::walk_block(this, block));
    }

    /// create a scope for fn parameters
    fn visit_fn(&mut self, sig: &'ast FnSig, body: Option<&'ast Expr>) {
        self.with_val_scope(|this| ast::walk_fn(this, sig, body));
    }

    fn visit_closure(&mut self, name: Option<Ident>, sig: &'ast FnSig, body: &'ast Expr) {
        self.with_val_scope(|this| ast::walk_closure(this, name, sig, body));
    }

    fn visit_item(&mut self, item: &'ast Item) {
        self.resolve_item(item)
    }

    fn visit_foreign_item(&mut self, item: &'ast ForeignItem) {
        self.resolve_foreign_item(item);
    }

    fn visit_let(&mut self, l @ Let { init, .. }: &'ast Let) {
        if let Some(expr) = init {
            if expr.is_named_closure() {
                self.resolver.emit_error(l.span, ResolutionError::BindingToNamedClosure);
            }
        }
        ast::walk_let(self, l)
    }

    fn visit_pattern(&mut self, pattern: &'ast Pattern) {
        self.resolve_pattern(pattern);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        match &expr.kind {
            ExprKind::Struct(path, _) | ExprKind::Path(path) => self.resolve_path(path, NS::Value),
            ExprKind::Closure(name, _sig, _body) =>
                if let Some(ident) = name {
                    self.def_val(*ident, Res::Local(expr.id))
                },
            _ => {}
        };
        ast::walk_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &'ast Ty) {
        match &ty.kind {
            TyKind::Path(path) => self.resolve_path(path, NS::Type),
            TyKind::ImplicitSelf => {
                let res = self.scopes[NS::Type]
                    .lookup(Ident::unspanned(kw::USelf))
                    .cloned()
                    .unwrap_or_else(|| {
                        self.emit_error(ty.span, ResolutionError::SelfParameterInFreeFunction)
                    });
                self.resolver.resolve_node(ty.id, res);
            }
            _ => {}
        };
        ast::walk_ty(self, ty);
    }
}

impl<'a> Resolver<'a> {
    pub fn late_resolve(&mut self, prog: &Ast) {
        let mut visitor = LateResolver::new(self);
        visitor.visit_ast(prog);
    }
}

impl<'r> Deref for LateResolver<'_, 'r, '_> {
    type Target = Resolver<'r>;

    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl<'r> DerefMut for LateResolver<'_, 'r, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

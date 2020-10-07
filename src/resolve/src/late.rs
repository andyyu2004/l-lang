use crate::*;
use ast::*;
use index::Idx;
use ir::{ModuleId, ParamIdx, Res, ROOT_MODULE};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct LateResolver<'a, 'r, 'ast> {
    resolver: &'a mut Resolver<'r>,
    scopes: PerNS<Scopes<Res<NodeId>>>,
    current_module: Vec<ModuleId>,
    pd: &'ast PhantomData<()>,
}

impl<'a, 'r, 'ast> LateResolver<'a, 'r, 'ast> {
    pub fn new(resolver: &'a mut Resolver<'r>) -> Self {
        Self {
            resolver,
            pd: &PhantomData,
            scopes: Default::default(),
            current_module: vec![ROOT_MODULE],
        }
    }

    pub fn with_module<R>(&mut self, module: ModuleId, f: impl FnOnce(&mut Self) -> R) -> R {
        self.current_module.push(module);
        let ret = f(self);
        self.current_module.pop();
        ret
    }

    pub fn with_val_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
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

    pub(super) fn def_val(&mut self, ident: Ident, res: Res<NodeId>) {
        self.scopes[NS::Value].def(ident, res);
    }

    fn resolve_pattern(&mut self, pat: &'ast Pattern) {
        PatternResolutionCtx::new(self).resolve_pattern(pat);
    }

    fn curr_module(&self) -> ModuleId {
        self.current_module.last().copied().unwrap()
    }

    /// searches for an item with name = `ident` in the current module
    fn try_resolve_item(&self, ident: Ident) -> Option<Res<NodeId>> {
        self.resolver.resolve_item(self.curr_module(), ident)
    }

    /// search for a local variable in the scopes otherwise look for a resolution to an item
    fn resolve_var(&mut self, ident: Ident) -> Option<Res<NodeId>> {
        match self.scopes[NS::Value].lookup(&ident) {
            Some(&res) => Some(res),
            None => self.try_resolve_item(ident),
        }
    }

    fn resolve_item(&mut self, item: &'ast Item) {
        match &item.kind {
            ItemKind::Fn(_, g, _) => self.with_generics(g, |r| ast::walk_item(r, item)),
            ItemKind::Enum(g, _) | ItemKind::Struct(g, _) => self.resolve_adt(g, item),
            ItemKind::Impl { generics, trait_path, self_ty, items } =>
                self.resolve_impl(generics, trait_path.as_ref(), self_ty, items),
            ItemKind::Extern(_) => self.with_val_scope(|r| ast::walk_item(r, item)),
        }
    }

    fn with_self_ty<R>(&mut self, _ty: &'ast Ty, f: impl FnOnce(&mut Self) -> R) -> R {
        self.with_ty_scope(|this| {
            this.scopes[NS::Type].def(Ident::unspanned(sym::USELF), Res::SelfTy);
            f(this)
        })
    }

    fn resolve_impl(
        &mut self,
        generics: &'ast Generics,
        trait_path: Option<&'ast Path>,
        self_ty: &'ast Ty,
        items: &'ast [Box<AssocItem>],
    ) {
        self.with_generics(generics, |this| {
            this.with_self_ty(self_ty, |this| {
                if let Some(path) = trait_path {
                    this.resolve_path(path, NS::Type);
                }
                this.visit_ty(self_ty);
                if trait_path.is_some() {
                    todo!()
                }
                for item in items {
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

    /// `id` belongs to the `Ty` or `Expr`
    pub(super) fn resolve_path(&mut self, path: &'ast Path, ns: NS) {
        let res = match ns {
            NS::Value => self.resolve_val_path(path),
            NS::Type => self.resolve_ty_path(path),
        };
        self.resolve_node(path.id, res);
    }

    fn resolve_val_path(&mut self, path: &'ast Path) -> Res<NodeId> {
        self.resolve_val_path_segments(path, &path.segments)
    }

    fn resolve_module(&mut self, ident: Ident) -> Option<ModuleId> {
        self.resolver.find_module(self.curr_module(), ident)
    }

    fn resolve_val_path_segments(
        &mut self,
        path: &'ast Path,
        segments: &'ast [PathSegment],
    ) -> Res<NodeId> {
        match &segments {
            [] => panic!("empty val path"),
            [segment] => self.resolve_path_segment(path, segment, NS::Value),
            [segment, xs @ ..] => match self.resolve_module(segment.ident) {
                Some(module) =>
                    self.with_module(module, |this| this.resolve_val_path_segments(path, xs)),
                None =>
                    return self.resolver.emit_error(
                        path.span,
                        ResolutionError::UnresolvedPath(segment.clone(), path.clone()),
                    ),
            },
        }
    }

    fn resolve_val_path_segment(
        &mut self,
        path: &'ast Path,
        segment: &'ast PathSegment,
    ) -> Res<NodeId> {
        self.resolve_var(segment.ident).unwrap_or_else(|| {
            let err = ResolutionError::UnresolvedPath(segment.clone(), path.clone());
            self.resolver.emit_error(path.span, err)
        })
    }

    fn resolve_ty_path(&mut self, path: &'ast Path) -> Res<NodeId> {
        match path.segments.as_slice() {
            [] => panic!("empty ty path"),
            [segment] => self.resolve_path_segment(path, segment, NS::Type),
            [xs @ .., segment] => todo!(),
        }
    }

    fn resolve_path_segment(
        &mut self,
        path: &'ast Path,
        segment: &'ast PathSegment,
        ns: NS,
    ) -> Res<NodeId> {
        self.visit_path_segment(segment);
        match ns {
            NS::Value => self.resolve_val_path_segment(path, segment),
            NS::Type => self.resolve_ty_path_segment(path, segment),
        }
    }

    fn resolve_ty_path_segment(
        &mut self,
        path: &'ast Path,
        segment: &'ast PathSegment,
    ) -> Res<NodeId> {
        if let Some(&res) = self.scopes[NS::Type].lookup(&segment.ident) {
            res
        } else if let Some(res) = self.try_resolve_item(segment.ident) {
            res
        } else if let Some(&ty) = self.resolver.primitive_types.get(&segment.ident.symbol) {
            Res::PrimTy(ty)
        } else {
            self.emit_error(path.span, ResolutionError::UnresolvedType(path.clone()))
        }
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
            _ => {}
        };
        ast::walk_ty(self, ty);
    }
}

impl<'a> Resolver<'a> {
    pub fn late_resolve(&mut self, prog: &Prog) {
        let mut visitor = LateResolver::new(self);
        visitor.visit_prog(prog);
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

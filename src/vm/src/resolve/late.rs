use super::{Resolver, Scope, Scopes};
use crate::ast::{self, *};
use crate::error::ResolutionError;
use crate::ir::{DefKind, ModuleId, ParamIdx, PerNS, Res, NS, ROOT_MODULE};
use indexed_vec::Idx;
use std::marker::PhantomData;

struct LateResolutionVisitor<'a, 'r, 'ast> {
    resolver: &'a mut Resolver<'r>,
    scopes: PerNS<Scopes<Res<NodeId>>>,
    current_module: Vec<ModuleId>,
    pd: &'ast PhantomData<()>,
}

impl<'a, 'r, 'ast> LateResolutionVisitor<'a, 'r, 'ast> {
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

    fn def_val(&mut self, ident: Ident, res: Res<NodeId>) {
        self.scopes[NS::Value].def(ident, res);
    }

    fn resolve_pattern(&mut self, pat: &'ast Pattern) {
        match pat.kind {
            PatternKind::Ident(ident, ..) => self.def_val(ident, Res::Local(pat.id)),
            PatternKind::Wildcard | PatternKind::Tuple(_) | PatternKind::Paren(_) => {}
        }
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
        }
    }

    fn resolve_adt(&mut self, generics: &'ast Generics, item: &'ast Item) {
        self.with_generics(generics, |this| ast::walk_item(this, item))
    }

    /// `id` belongs to the `Ty` or `Expr`
    fn resolve_path(&mut self, id: NodeId, path: &'ast Path, ns: NS) {
        let res = match ns {
            NS::Value => self.resolve_val_path(path),
            NS::Type => self.resolve_ty_path(path),
        };
        self.resolver.resolve_node(id, res);
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
        match segments {
            [] => panic!("empty val path"),
            &[segment] => self.resolve_var(segment.ident).unwrap_or_else(|| {
                let err = ResolutionError::UnresolvedPath(segment, path.clone());
                self.resolver.emit_error(path.span, err)
            }),
            [segment, xs @ ..] => match self.resolve_module(segment.ident) {
                Some(module) =>
                    self.with_module(module, |this| this.resolve_val_path_segments(path, xs)),
                None =>
                    return self.resolver.emit_error(
                        path.span,
                        ResolutionError::UnresolvedPath(*segment, path.clone()),
                    ),
            },
        }
    }

    fn resolve_ty_path(&mut self, path: &'ast Path) -> Res<NodeId> {
        match path.segments.as_slice() {
            [] => panic!("empty ty path"),
            [segment] => self.resolve_ty_path_segment(path, segment),
            [xs @ .., segment] => todo!(),
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
            self.resolver.emit_error(path.span, ResolutionError::UnresolvedType(path.clone()))
        }
    }

    /// bring each parameter into scope
    fn resolve_params(&mut self, params: &'ast [Param]) {
        params.iter().for_each(|param| {
            self.resolve_pattern(&param.pattern);
            self.visit_ty(&param.ty);
        })
    }
}

impl<'a, 'ast> ast::Visitor<'ast> for LateResolutionVisitor<'a, '_, 'ast> {
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
        ast::walk_pat(self, pattern);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        match &expr.kind {
            ExprKind::Struct(path, _) | ExprKind::Path(path) =>
                self.resolve_path(expr.id, path, NS::Value),
            ExprKind::Closure(name, sig, body) =>
                if let Some(ident) = name {
                    self.def_val(*ident, Res::Local(expr.id))
                },
            _ => {}
        };
        ast::walk_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &'ast Ty) {
        match &ty.kind {
            TyKind::Path(path) => self.resolve_path(ty.id, path, NS::Type),
            _ => {}
        };
        ast::walk_ty(self, ty);
    }
}

impl<'a> Resolver<'a> {
    pub fn late_resolve_prog(&mut self, prog: &Prog) {
        let mut visitor = LateResolutionVisitor::new(self);
        visitor.visit_prog(prog);
    }
}

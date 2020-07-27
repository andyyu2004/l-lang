use super::{Resolver, Scope, Scopes};
use crate::ast::{self, *};
use crate::error::{ResolutionError, ResolutionResult};
use crate::ir::{DefKind, PerNS, Res, NS};
use std::marker::PhantomData;

struct LateResolutionVisitor<'a, 'r, 'ast> {
    resolver: &'r mut Resolver,
    scopes: PerNS<Scopes<Res<NodeId>>>,
    pd: &'a PhantomData<&'ast ()>,
}

impl<'a, 'r, 'ast> LateResolutionVisitor<'a, 'r, 'ast> {
    pub fn new(resolver: &'r mut Resolver) -> Self {
        Self { resolver, pd: &PhantomData, scopes: Default::default() }
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
                let res = Res::Def(this.resolver.def_id(param.id), DefKind::TyParam);
                this.scopes[NS::Type].def(param.ident, res)
            }
            f(this)
        })
    }

    fn resolve_pattern(&mut self, pat: &'ast Pattern) {
        match &pat.kind {
            PatternKind::Ident(ident, _) => self.scopes[NS::Value].def(*ident, Res::Local(pat.id)),
            PatternKind::Wildcard | PatternKind::Tuple(_) | PatternKind::Paren(_) => {}
        }
    }

    /// search for a local variable in the scopes otherwise look for a resolution to an item
    fn resolve_value(&mut self, ident: Ident) -> Option<Res<NodeId>> {
        match self.scopes[NS::Value].lookup(&ident) {
            Some(&res) => Some(res),
            None => self.resolver.resolve_item(ident),
        }
    }

    fn resolve_item(&mut self, item: &'ast Item) {
        match &item.kind {
            ItemKind::Fn(_, g, _) => self.with_generics(g, |r| ast::walk_item(r, item)),
        }
    }

    fn resolve_path(&mut self, path: &'ast Path, ns: NS) -> ResolutionResult<()> {
        if path.segments.len() > 1 {
            unimplemented!()
        }
        let segment = &path.segments[0];
        let res = match ns {
            NS::Value => self
                .resolve_value(segment.ident)
                .ok_or_else(|| ResolutionError::unbound_variable(path.clone()))?,
            // just lookup primitive type for now as there are no other potential types
            NS::Type => self.resolve_ty_path(path)?,
        };
        self.resolver.resolve_node(segment.id, res);
        Ok(())
    }

    fn resolve_ty_path(&mut self, path: &'ast Path) -> ResolutionResult<Res<NodeId>> {
        let segment = &path.segments[0];
        if let Some(&res) = self.scopes[NS::Type].lookup(&segment.ident) {
            Ok(res)
        } else if let Some(&ty) = self.resolver.primitive_types.get(&segment.ident.symbol) {
            Ok(Res::PrimTy(ty))
        } else {
            Err(ResolutionError::unknown_type(path.clone()))
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

    fn visit_lambda(&mut self, sig: &'ast FnSig, expr: &'ast Expr) {
        self.with_val_scope(|this| ast::walk_lambda(this, sig, expr));
    }

    fn visit_item(&mut self, item: &'ast Item) {
        self.resolve_item(item)
    }

    fn visit_pattern(&mut self, pattern: &'ast Pattern) {
        self.resolve_pattern(pattern);
        ast::walk_pat(self, pattern);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        match &expr.kind {
            ExprKind::Path(path) => self.resolve_path(path, NS::Value).unwrap(),
            _ => {}
        };
        ast::walk_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &'ast Ty) {
        match &ty.kind {
            TyKind::Path(path) => return self.resolve_path(path, NS::Type).unwrap(),
            _ => {}
        };
        ast::walk_ty(self, ty);
    }
}

impl Resolver {
    crate fn late_resolve_prog(&mut self, prog: &Prog) {
        let mut visitor = LateResolutionVisitor::new(self);
        visitor.visit_prog(prog);
    }
}

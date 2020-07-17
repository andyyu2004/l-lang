use super::{Resolver, Scope, Scopes};
use crate::ast::{self, *};
use crate::error::{ResolutionError, ResolutionResult};
use crate::ir::{PerNS, Res, NS};
use std::marker::PhantomData;

struct LateResolutionVisitor<'a, 'r, 'ast> {
    resolver: &'r mut Resolver,
    scopes: PerNS<Scopes<Res<NodeId>>>,
    pd: &'a PhantomData<&'ast ()>,
}

impl<'a, 'r, 'ast> LateResolutionVisitor<'a, 'r, 'ast> {
    pub fn new(resolver: &'r mut Resolver) -> Self {
        Self { resolver, pd: &PhantomData, scopes: Self::mk_global_scope() }
    }

    /// creates a scope that include primitive types
    fn mk_global_scope() -> PerNS<Scopes<Res<NodeId>>> {
        let scopes = PerNS::default();
        scopes
    }

    pub fn with_val_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.scopes[NS::Value].push(Scope::default());
        let ret = f(self);
        self.scopes[NS::Value].pop();
        ret
    }

    fn resolve_pattern(&mut self, pat: &'ast Pattern) {
        match &pat.kind {
            PatternKind::Ident(ident, _) => self.scopes[NS::Value].def(*ident, Res::Local(pat.id)),
            PatternKind::Wildcard | PatternKind::Tuple(_) | PatternKind::Paren(_) => {}
        }
    }

    fn resolve_path(&mut self, path: &'ast Path, ns: NS) -> ResolutionResult<()> {
        if path.segments.len() > 1 {
            unimplemented!()
        }
        let segment = &path.segments[0];
        let res = match ns {
            NS::Value => *self.scopes[ns]
                .lookup(&segment.ident)
                .ok_or_else(|| ResolutionError::unbound_variable(path.clone()))?,
            // just lookup primitive type for now as there are no other potential types
            NS::Type => Res::PrimTy(
                *self
                    .resolver
                    .primitive_types
                    .get(&segment.ident.symbol)
                    .ok_or_else(|| ResolutionError::unknown_type(path.clone()))?,
            ),
        };
        self.resolver.resolve_node(segment.id, res);
        Ok(())
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
        self.with_val_scope(|resolver| ast::walk_block(resolver, block));
    }

    fn visit_lambda(&mut self, sig: &'ast FnSig, expr: &'ast Expr) {
        self.with_val_scope(|resolver| {
            resolver.resolve_params(&sig.inputs);
            if let Some(ty) = &sig.output {
                resolver.visit_ty(ty)
            }
            resolver.visit_expr(expr);
        });
    }

    fn visit_let(&mut self, Let { pat, ty, init, .. }: &'ast Let) {
        self.resolve_pattern(pat);
        ast::walk_pat(self, pat);
        ty.as_ref().map(|ty| self.visit_ty(ty));
        init.as_ref().map(|expr| self.visit_expr(expr));
    }

    fn visit_item(&mut self, item: &'ast Item) {
        self.resolver.create_def(item.id);
        ast::walk_item(self, item);
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

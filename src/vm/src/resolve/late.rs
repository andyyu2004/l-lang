use super::{Resolver, Scope, Scopes};
use crate::ast::{self, *};
use crate::error::{ResolutionError, ResolutionResult};
use crate::ir::{DefKind, ParamIdx, PerNS, Res, NS};
use indexed_vec::Idx;
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

    /// search for a local variable in the scopes otherwise look for a resolution to an item
    fn resolve_var(&mut self, ident: Ident) -> Option<Res<NodeId>> {
        match self.scopes[NS::Value].lookup(&ident) {
            Some(&res) => Some(res),
            None => self.resolver.resolve_item(ident),
        }
    }

    fn resolve_item(&mut self, item: &'ast Item) {
        match &item.kind {
            ItemKind::Fn(_, g, _) => self.with_generics(g, |r| ast::walk_item(r, item)),
            ItemKind::Enum(g, _) | ItemKind::Struct(g, _) => self.resolve_adt(g, item),
        }
    }

    fn resolve_adt(&mut self, generics: &'ast Generics, item: &'ast Item) {
        self.with_generics(generics, |this| {
            // let res = Res::Def(item.id.def, DefKind::Struct);
            // this.scopes[NS::Type].def(item.ident, res);
            ast::walk_item(this, item);
        })
    }

    fn resolve_path(&mut self, path: &'ast Path, ns: NS) -> ResolutionResult<()> {
        if path.segments.len() > 1 {
            unimplemented!()
        }
        let segment = &path.segments[0];
        let res = match ns {
            NS::Value => self
                .resolve_var(segment.ident)
                .ok_or_else(|| ResolutionError::unbound_variable(path.clone()))?,
            // just lookup primitive type for now as there are no other potential types
            NS::Type => self.resolve_ty_path(path)?,
        };
        self.resolver.resolve_node(segment.id, res);
        Ok(())
    }

    fn resolve_ty_path(&mut self, path: &'ast Path) -> ResolutionResult<Res<NodeId>> {
        match path.segments.as_slice() {
            [] => panic!("empty path"),
            [segment] => self.resolve_ty_path_segment(path, segment),
            [xs @ .., segment] => todo!(),
        }
    }

    fn resolve_ty_path_segment(
        &mut self,
        path: &'ast Path,
        segment: &'ast PathSegment,
    ) -> ResolutionResult<Res<NodeId>> {
        if let Some(&res) = self.scopes[NS::Type].lookup(&segment.ident) {
            Ok(res)
        } else if let Some(res) = self.resolver.resolve_item(segment.ident) {
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

    fn visit_closure(&mut self, name: Option<Ident>, sig: &'ast FnSig, body: &'ast Expr) {
        self.with_val_scope(|this| ast::walk_closure(this, name, sig, body));
    }

    fn visit_item(&mut self, item: &'ast Item) {
        self.resolve_item(item)
    }

    fn visit_let(&mut self, l @ Let { init, .. }: &'ast Let) {
        if let Some(expr) = init {
            if expr.is_named_closure() {
                panic!("let binding to named closure")
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
            // TODO better error handling
            ExprKind::Struct(path, _) | ExprKind::Path(path) =>
                self.resolve_path(path, NS::Value).unwrap(),
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
            TyKind::Path(path) => self.resolve_path(path, NS::Type).unwrap(),
            _ => {}
        };
        ast::walk_ty(self, ty);
    }
}

impl Resolver {
    pub fn late_resolve_prog(&mut self, prog: &Prog) {
        let mut visitor = LateResolutionVisitor::new(self);
        visitor.visit_prog(prog);
    }
}

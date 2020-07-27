use super::*;

/// traverse the ast; each function can be overridden.
/// by default, just recursively visits each substructure
crate trait Visitor<'ast>: Sized {
    fn visit_item(&mut self, item: &'ast Item) {
        walk_item(self, item)
    }

    fn visit_prog(&mut self, prog: &'ast Prog) {
        prog.items.iter().for_each(|item| self.visit_item(item));
    }

    fn visit_generics(&mut self, generics: &'ast Generics) {
        walk_generics(self, generics)
    }

    fn visit_ty_param(&mut self, ty_param: &'ast TyParam) {
        self.visit_ident(ty_param.ident)
    }

    fn visit_vis(&mut self, vis: &'ast Visibility) {
    }

    fn visit_fn(&mut self, sig: &'ast FnSig, body: Option<&'ast Expr>) {
        walk_fn(self, sig, body);
    }

    fn visit_block(&mut self, block: &'ast Block) {
        walk_block(self, block);
    }

    /// visit the initializer first in case the same pattern is referenced in the initializer
    /// let x = 1;
    /// let x = x;
    /// this will only behave correctly if the pattern is resolved after the initializer
    fn visit_let(&mut self, Let { pat, ty, init, .. }: &'ast Let) {
        init.as_ref().map(|expr| self.visit_expr(expr));
        self.visit_pattern(pat);
        ty.as_ref().map(|ty| self.visit_ty(ty));
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        walk_expr(self, expr)
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        match &stmt.kind {
            StmtKind::Let(l) => self.visit_let(l),
            StmtKind::Expr(expr) => self.visit_expr(expr),
            StmtKind::Semi(expr) => self.visit_expr(expr),
        }
    }

    fn visit_fn_sig(&mut self, sig: &'ast FnSig) {
        sig.inputs.iter().for_each(|p| self.visit_param(p));
        if let Some(ret_ty) = &sig.output {
            self.visit_ty(ret_ty)
        }
    }

    fn visit_lambda(&mut self, sig: &'ast FnSig, expr: &'ast Expr) {
        walk_lambda(self, sig, expr)
    }

    fn visit_param(&mut self, param: &'ast Param) {
        self.visit_pattern(&param.pattern);
        self.visit_ty(&param.ty);
    }

    fn visit_pattern(&mut self, pattern: &'ast Pattern) {
        walk_pat(self, pattern);
    }

    fn visit_path(&mut self, path: &'ast Path) {
        path.segments.iter().for_each(|seg| self.visit_path_segment(seg))
    }

    fn visit_path_segment(&mut self, segment: &'ast PathSegment) {
        self.visit_ident(segment.ident)
    }

    fn visit_ty(&mut self, ty: &'ast Ty) {
        walk_ty(self, ty)
    }

    fn visit_ident(&mut self, _ident: Ident) {
    }
}

crate fn walk_fn<'ast>(
    visitor: &mut impl Visitor<'ast>,
    sig: &'ast FnSig,
    body: Option<&'ast Expr>,
) {
    visitor.visit_fn_sig(sig);
    body.as_ref().map(|body| visitor.visit_expr(body));
}

crate fn walk_block<'ast>(visitor: &mut impl Visitor<'ast>, block: &'ast Block) {
    block.stmts.iter().for_each(|stmt| visitor.visit_stmt(stmt))
}

crate fn walk_expr<'ast>(visitor: &mut impl Visitor<'ast>, expr: &'ast Expr) {
    match &expr.kind {
        ExprKind::Lit(_) => {}
        ExprKind::Unary(_, expr) => visitor.visit_expr(expr),
        ExprKind::Paren(expr) => visitor.visit_expr(expr),
        ExprKind::Block(block) => visitor.visit_block(block),
        ExprKind::Path(path) => visitor.visit_path(path),
        ExprKind::Tuple(xs) => xs.iter().for_each(|expr| visitor.visit_expr(expr)),
        ExprKind::Lambda(sig, expr) => visitor.visit_lambda(sig, expr),
        ExprKind::Call(f, args) => {
            visitor.visit_expr(f);
            args.iter().for_each(|expr| visitor.visit_expr(expr));
        }
        ExprKind::If(c, l, r) => {
            visitor.visit_expr(c);
            visitor.visit_block(l);
            r.as_ref().map(|expr| visitor.visit_expr(expr));
        }
        ExprKind::Bin(_, l, r) => {
            visitor.visit_expr(l);
            visitor.visit_expr(r);
        }
    }
}

crate fn walk_generics<'ast>(visitor: &mut impl Visitor<'ast>, generics: &'ast Generics) {
    generics.params.iter().for_each(|p| visitor.visit_ty_param(p));
}

crate fn walk_lambda<'ast>(visitor: &mut impl Visitor<'ast>, sig: &'ast FnSig, expr: &'ast Expr) {
    visitor.visit_fn_sig(sig);
    visitor.visit_expr(expr);
}

crate fn walk_pat<'ast>(visitor: &mut impl Visitor<'ast>, pat: &'ast Pattern) {
    match &pat.kind {
        PatternKind::Wildcard => {}
        PatternKind::Ident(ident, pat) => {
            visitor.visit_ident(*ident);
            pat.as_ref().map(|p| visitor.visit_pattern(p));
        }
        PatternKind::Paren(pat) => visitor.visit_pattern(pat),
        PatternKind::Tuple(pats) => pats.iter().for_each(|p| visitor.visit_pattern(p)),
    }
}

crate fn walk_ty<'ast>(visitor: &mut impl Visitor<'ast>, ty: &'ast Ty) {
    match &ty.kind {
        TyKind::Array(ty) => visitor.visit_ty(ty),
        TyKind::Tuple(tys) => tys.iter().for_each(|ty| visitor.visit_ty(ty)),
        TyKind::Paren(ty) => visitor.visit_ty(ty),
        TyKind::Path(path) => visitor.visit_path(path),
        TyKind::Fn(params, ret) => {
            params.iter().for_each(|ty| visitor.visit_ty(ty));
            ret.as_ref().map(|ty| visitor.visit_ty(ty));
        }
        TyKind::Infer => {}
    }
}

crate fn walk_item<'ast>(visitor: &mut impl Visitor<'ast>, item: &'ast Item) {
    let Item { vis, ident, kind, .. } = &item;
    visitor.visit_vis(vis);
    visitor.visit_ident(*ident);
    match kind {
        ItemKind::Fn(sig, generics, body) => {
            visitor.visit_generics(generics);
            visitor.visit_fn(sig, body.as_deref())
        }
    }
}

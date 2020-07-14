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
    }

    fn visit_vis(&mut self, vis: &'ast Visibility) {
    }

    fn visit_fn(&mut self, sig: &'ast FnSig, body: Option<&'ast Block>) {
        self.visit_fn_sig(sig);
        if let Some(body) = body {
            self.visit_block(body);
        }
    }

    fn visit_block(&mut self, block: &'ast Block) {
        walk_block(self, block);
    }

    fn visit_let(&mut self, Let { pat, ty, init, .. }: &'ast Let) {
        self.visit_pattern(pat);
        ty.as_ref().map(|ty| self.visit_ty(ty));
        init.as_ref().map(|expr| self.visit_expr(expr));
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

    fn visit_param(&mut self, param: &'ast Param) {
        self.visit_pattern(&param.pattern);
        self.visit_ty(&param.ty);
    }

    fn visit_pattern(&mut self, pattern: &'ast Pattern) {
        match &pattern.kind {
            PatternKind::Wildcard => {}
            PatternKind::Ident(ident, pat) => {
                self.visit_ident(*ident);
                if let Some(pat) = pat {
                    self.visit_pattern(pat)
                }
            }
            PatternKind::Paren(pat) => self.visit_pattern(pat),
        }
    }

    fn visit_path(&mut self, path: &'ast Path) {
        path.segments.iter().for_each(|seg| self.visit_path_segment(seg))
    }

    fn visit_path_segment(&mut self, segment: &'ast PathSegment) {
        self.visit_ident(segment.ident)
    }

    fn visit_ty(&mut self, ty: &'ast Ty) {
        match &ty.kind {
            TyKind::Array(ty) => self.visit_ty(ty),
            TyKind::Tuple(tys) => tys.iter().for_each(|ty| self.visit_ty(ty)),
            TyKind::Paren(ty) => self.visit_ty(ty),
            TyKind::Path(path) => self.visit_path(path),
        }
    }

    fn visit_ident(&mut self, _ident: Ident) {
    }
}

crate fn walk_block<'ast>(visitor: &mut impl Visitor<'ast>, block: &'ast Block) {
    block.stmts.iter().for_each(|stmt| visitor.visit_stmt(stmt))
}

crate fn walk_expr<'ast>(visitor: &mut impl Visitor<'ast>, expr: &'ast Expr) {
    match &expr.kind {
        ExprKind::Lit(_) => {}
        ExprKind::Bin(_, l, r) => {
            visitor.visit_expr(l);
            visitor.visit_expr(r);
        }
        ExprKind::Unary(_, expr) => visitor.visit_expr(expr),
        ExprKind::Paren(expr) => visitor.visit_expr(expr),
        ExprKind::Block(block) => visitor.visit_block(block),
        ExprKind::Path(path) => visitor.visit_path(path),
        ExprKind::Tuple(_) => todo!(),
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

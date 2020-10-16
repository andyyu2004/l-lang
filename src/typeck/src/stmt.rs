use crate::FnCtx;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn check_stmt(&mut self, stmt: &ir::Stmt) {
        match &stmt.kind {
            ir::StmtKind::Let(l) => self.check_let_stmt(l),
            ir::StmtKind::Expr(expr) => {
                self.check_expr(expr);
            }
            ir::StmtKind::Semi(expr) => {
                self.check_expr(expr);
            }
        }
    }

    pub fn check_let_stmt(&mut self, l: &ir::Let) {
        let ty =
            l.init.map(|expr| self.check_expr(expr)).unwrap_or_else(|| self.new_infer_var(l.span));
        l.ty.iter().for_each(|t| self.equate(l.span, self.lower_ty(t), ty));
        let pat_ty = self.check_pat(l.pat, ty);
        self.equate(l.span, ty, pat_ty);
    }
}

use crate::{FnCtx, TyConv};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn check_stmt(&mut self, stmt: &ir::Stmt<'tcx>) {
        match &stmt.kind {
            ir::StmtKind::Let(l) => self.check_let_stmt(l),
            ir::StmtKind::Semi(expr) => {
                self.check_expr(expr);
            }
            ir::StmtKind::Expr(expr) => unreachable!(),
        }
    }

    pub fn check_let_stmt(&mut self, l: &ir::Let<'tcx>) {
        let ty =
            l.init.map(|expr| self.check_expr(expr)).unwrap_or_else(|| self.new_infer_var(l.span));
        l.ty.iter().for_each(|t| self.unify(l.span, self.ir_ty_to_ty(t), ty));
        let pat_ty = self.check_pat(l.pat, ty);
        self.unify(l.span, ty, pat_ty);
    }
}

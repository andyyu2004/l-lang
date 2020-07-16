use super::FnCtx;
use crate::ir;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn check_let_stmt(&mut self, l: &ir::Let) {
        // type of the entire binding
        let ty = l.ty.map(|ty| self.lower_ty(ty)).unwrap_or(self.new_infer_var());
        // type of the pattern
        let pat_ty = self.check_pat(l.pat, ty);
        self.expect_eq(l.span, ty, pat_ty);
        // type of the init expression
        let init_ty = l
            .init
            .as_ref()
            .map(|expr| self.check_expr(expr))
            .map(|init_ty| self.expect_eq(l.span, init_ty, ty));
        // we want pty = ty = initty
    }

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
}

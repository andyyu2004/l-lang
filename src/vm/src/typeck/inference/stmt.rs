use super::FnCtx;
use crate::ir;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn check_stmt(&mut self, stmt: &ir::Stmt) {
        match &stmt.kind {
            ir::StmtKind::Ret(ret_expr) => self.check_ret_stmt(stmt, ret_expr.as_deref()),
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
        let ty = l.init.map(|expr| self.check_expr(expr)).unwrap_or_else(|| self.new_infer_var());
        l.ty.map(|t| self.unify(l.span, self.lower_ty(t), ty));
        let pat_ty = self.check_pat(l.pat, ty);
        self.unify(l.span, ty, pat_ty);
    }

    /// return expressions have the type of the expression that follows the return
    fn check_ret_stmt(&mut self, stmt: &ir::Stmt, ret_expr: Option<&ir::Expr>) {
        let ty = ret_expr.map(|expr| self.check_expr(expr)).unwrap_or(self.tcx.types.unit);
        self.unify(stmt.span, self.expected_ret_ty, ty);
    }
}

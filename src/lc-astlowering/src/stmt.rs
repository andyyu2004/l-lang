use super::AstLoweringCtx;
use lc_ast::*;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    pub fn lower_stmt(&mut self, stmt: &Stmt) -> &'ir ir::Stmt<'ir> {
        self.arena.alloc(self.lower_stmt_inner(stmt))
    }

    pub fn lower_stmt_inner(&mut self, stmt: &Stmt) -> ir::Stmt<'ir> {
        let kind = match &stmt.kind {
            StmtKind::Let(l) => ir::StmtKind::Let(self.lower_let_stmt(l)),
            StmtKind::Expr(expr) => ir::StmtKind::Expr(self.lower_expr(expr)),
            StmtKind::Semi(expr) => ir::StmtKind::Semi(self.lower_expr(expr)),
        };
        ir::Stmt { id: self.lower_node_id(stmt.id), span: stmt.span, kind }
    }

    pub fn lower_let_stmt(&mut self, l: &Let) -> &'ir ir::Let<'ir> {
        let &Let { id, span, ref pat, ref ty, ref init } = l;
        self.arena.alloc(ir::Let {
            id: self.lower_node_id(id),
            span,
            pat: self.lower_pattern(pat),
            ty: ty.as_ref().map(|ty| self.lower_ty(ty)),
            init: init.as_ref().map(|init| self.lower_expr(init)),
        })
    }
}

use super::LoweringCtx;
use crate::ast::*;
use crate::ir;

impl<'ir> LoweringCtx<'ir> {
    fn lower_stmts(&mut self, stmts: &[Box<Stmt>]) -> &'ir [ir::Stmt<'ir>] {
        self.arena.alloc_from_iter(stmts.iter().map(|x| self.lower_stmt_inner(x)))
    }

    crate fn lower_stmt(&mut self, stmt: &Stmt) -> &'ir ir::Stmt<'ir> {
        self.arena.alloc(self.lower_stmt_inner(stmt))
    }

    crate fn lower_stmt_inner(&mut self, stmt: &Stmt) -> ir::Stmt<'ir> {
        let kind = match &stmt.kind {
            StmtKind::Let(_) => {}
            StmtKind::Expr(_) => {}
            StmtKind::Semi(_) => {}
            StmtKind::Empty => {}
        };
        // ir::Stmt::new(stmt.span, kind)
        todo!()
    }
}

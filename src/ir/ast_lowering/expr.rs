use super::LoweringCtx;
use crate::ast::*;
use crate::ir;

impl<'ir> LoweringCtx<'ir> {
    fn lower_exprs(&mut self, exprs: &[Box<Expr>]) -> &'ir [ir::Expr<'ir>] {
        self.arena
            .alloc_from_iter(exprs.iter().map(|x| self.lower_expr_inner(x)))
    }

    crate fn lower_expr(&mut self, e: &Expr) -> &'ir ir::Expr<'ir> {
        self.arena.alloc(self.lower_expr_inner(e))
    }

    fn lower_expr_inner(&mut self, e: &Expr) -> ir::Expr<'ir> {
        let kind = match &e.kind {
            ExprKind::Lit(lit) => ir::ExprKind::Lit(*lit),
            ExprKind::Bin(op, l, r) => {
                ir::ExprKind::Bin(*op, self.lower_expr(&l), self.lower_expr(&r))
            }
            ExprKind::Unary(op, expr) => ir::ExprKind::Unary(*op, self.lower_expr(&expr)),
            ExprKind::Paren(expr) => ir::ExprKind::Paren(self.lower_expr(&expr)),
        };
        ir::Expr::new(e.span, kind)
    }
}

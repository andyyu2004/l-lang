use crate::AstLoweringCtx;
use ast::Lit;
use span::Span;

/// methods for constructing `ir` for desugaring purposes
impl<'a, 'ir> AstLoweringCtx<'a, 'ir> {
    crate fn mk_expr(&mut self, span: Span, kind: ir::ExprKind<'ir>) -> &'ir ir::Expr<'ir> {
        self.arena.alloc(ir::Expr { id: self.new_id(), span, kind })
    }

    crate fn mk_expr_bool(&mut self, span: Span, b: bool) -> &'ir ir::Expr<'ir> {
        self.mk_expr(span, ir::ExprKind::Lit(Lit::Bool(b)))
    }

    crate fn mk_pat_bool(&mut self, span: Span, b: bool) -> &'ir ir::Pattern<'ir> {
        let expr = self.mk_expr_bool(span, b);
        self.mk_pat(span, ir::PatternKind::Lit(expr))
    }

    crate fn mk_pat(&mut self, span: Span, kind: ir::PatternKind<'ir>) -> &'ir ir::Pattern<'ir> {
        self.arena.alloc(ir::Pattern { id: self.new_id(), span, kind })
    }

    crate fn mk_empty_block_expr(&mut self, span: Span) -> &'ir ir::Expr<'ir> {
        let block = self.mk_empty_block(span);
        self.mk_expr(span, ir::ExprKind::Block(block))
    }

    crate fn mk_ty_path(&mut self, span: Span, qpath: &'ir ir::QPath<'ir>) -> &'ir ir::Ty<'ir> {
        self.arena.alloc(ir::Ty { id: self.new_id(), span, kind: ir::TyKind::Path(qpath) })
    }

    crate fn mk_empty_block(&mut self, span: Span) -> &'ir ir::Block<'ir> {
        self.arena.alloc(ir::Block {
            id: self.new_id(),
            span,
            stmts: &[],
            is_unsafe: false,
            expr: None,
        })
    }

    crate fn mk_arm(
        &mut self,
        pat: &'ir ir::Pattern<'ir>,
        expr: &'ir ir::Expr<'ir>,
    ) -> ir::Arm<'ir> {
        ir::Arm { id: self.new_id(), span: pat.span.merge(expr.span), pat, guard: None, body: expr }
    }
}

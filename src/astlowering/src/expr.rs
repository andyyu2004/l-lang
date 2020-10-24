use crate::AstLoweringCtx;
use ast::*;
use itertools::Itertools;
use span::Span;
use std::array::IntoIter;

impl<'ir> AstLoweringCtx<'_, 'ir> {
    fn lower_exprs(&mut self, exprs: &[Box<Expr>]) -> &'ir [ir::Expr<'ir>] {
        self.arena.alloc_from_iter(exprs.iter().map(|x| self.lower_expr_inner(x)))
    }

    pub fn lower_expr(&mut self, expr: &Expr) -> &'ir ir::Expr<'ir> {
        self.arena.alloc(self.lower_expr_inner(expr))
    }

    fn lower_expr_inner(&mut self, expr: &Expr) -> ir::Expr<'ir> {
        let kind = match &expr.kind {
            ExprKind::Box(expr) => ir::ExprKind::Box(self.lower_expr(expr)),
            ExprKind::Lit(lit) => ir::ExprKind::Lit(*lit),
            ExprKind::Ret(expr) =>
                ir::ExprKind::Ret(expr.as_deref().map(|expr| self.lower_expr(expr))),
            ExprKind::Unary(op, expr) => ir::ExprKind::Unary(*op, self.lower_expr(&expr)),
            ExprKind::Paren(expr) => return self.lower_expr_inner(&expr),
            ExprKind::Block(block) => ir::ExprKind::Block(self.lower_block(block)),
            ExprKind::Path(path) => ir::ExprKind::Path(self.lower_qpath(path)),
            ExprKind::Tuple(xs) => ir::ExprKind::Tuple(self.lower_exprs(xs)),
            ExprKind::Bin(op, l, r) =>
                ir::ExprKind::Bin(*op, self.lower_expr(&l), self.lower_expr(&r)),
            ExprKind::Closure(_name, sig, expr) => {
                let lowered_sig = self.lower_fn_sig(sig);
                let body = self.lower_body(sig, expr);
                ir::ExprKind::Closure(lowered_sig, body)
            }
            ExprKind::Call(f, args) =>
                ir::ExprKind::Call(self.lower_expr(f), self.lower_exprs(args)),
            ExprKind::If(c, l, r) => self.lower_expr_if(expr.span, &c, &l, r.as_deref()),
            ExprKind::Struct(path, fields) => ir::ExprKind::Struct(
                self.lower_path(path),
                self.arena.alloc_from_iter(fields.iter().map(|f| self.lower_field(f))),
            ),
            ExprKind::Assign(l, r) => ir::ExprKind::Assign(self.lower_expr(l), self.lower_expr(r)),
            ExprKind::Field(expr, ident) => ir::ExprKind::Field(self.lower_expr(expr), *ident),
            ExprKind::Match(expr, arms) => ir::ExprKind::Match(
                self.lower_expr(expr),
                self.lower_arms(arms),
                ir::MatchSource::Match,
            ),
            ExprKind::Err => ir::ExprKind::Err,
        };

        ir::Expr { span: expr.span, id: self.lower_node_id(expr.id), kind }
    }

    fn lower_arms(&mut self, arms: &[Arm]) -> &'ir [ir::Arm<'ir>] {
        self.arena.alloc_from_iter(arms.iter().map(|arm| self.lower_arm(arm)))
    }

    fn lower_arm(&mut self, arm: &Arm) -> ir::Arm<'ir> {
        ir::Arm {
            id: self.lower_node_id(arm.id),
            span: arm.span,
            pat: self.lower_pattern(&arm.pat),
            guard: arm.guard.as_ref().map(|guard| self.lower_expr(guard)),
            body: self.lower_expr(&arm.body),
        }
    }

    fn lower_field(&mut self, field: &Field) -> ir::Field<'ir> {
        let &Field { id, span, ident, ref expr } = field;
        ir::Field { id: self.lower_node_id(id), span, ident, expr: self.lower_expr(expr) }
    }

    /// desugars into a match with a `true` branch and a wildcard branch
    /// an empty else branch desugars into an empty block
    /// this also works for typechecking as an if with no else will force the then branch to be of
    /// type unit as expected
    fn lower_expr_if(
        &mut self,
        span: Span,
        c: &Expr,
        l: &Block,
        r: Option<&Expr>,
    ) -> ir::ExprKind<'ir> {
        let scrutinee = self.lower_expr(c);
        // `then` branch
        let then_pat = self.mk_pat_bool(span, true);
        let then_block = self.lower_block(l);
        let then_expr = self.mk_expr(then_block.span, ir::ExprKind::Block(then_block));
        let then_arm = self.mk_arm(then_pat, then_expr);
        // `else` branch
        let else_pat = self.mk_pat(span, ir::PatternKind::Wildcard);
        let else_expr = match r {
            Some(expr) => self.lower_expr(expr),
            None => self.mk_empty_block_expr(span),
        };
        let else_arm = self.mk_arm(else_pat, else_expr);
        let arms = IntoIter::new([then_arm, else_arm]);
        ir::ExprKind::Match(scrutinee, self.alloc_from_iter(arms), ir::MatchSource::If)
    }

    pub(super) fn lower_block(&mut self, block: &Block) -> &'ir ir::Block<'ir> {
        let mut expr = None;
        let mut stmts = block.stmts.iter().map(|stmt| self.lower_stmt_inner(stmt)).collect_vec();
        if let Some(&ir::StmtKind::Expr(e)) = stmts.last().map(|s| &s.kind) {
            expr = Some(e);
            stmts.pop();
        }
        let ir_block = ir::Block {
            stmts: self.alloc_from_iter(stmts),
            id: self.lower_node_id(block.id),
            is_unsafe: block.is_unsafe,
            span: block.span,
            expr,
        };
        self.alloc(ir_block)
    }
}

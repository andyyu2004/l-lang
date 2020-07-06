use crate::ast;
use crate::span::Span;

#[derive(Debug)]
crate struct Expr<'ir> {
    pub span: Span,
    pub kind: ExprKind<'ir>,
}

impl<'ir> Expr<'ir> {
    pub fn new(span: Span, kind: ExprKind<'ir>) -> Self {
        Self { span, kind }
    }
}

#[derive(Debug)]
crate enum ExprKind<'ir> {
    Lit(ast::Lit),
    Bin(ast::BinOp, &'ir Expr<'ir>, &'ir Expr<'ir>),
    Unary(ast::UnaryOp, &'ir Expr<'ir>),
}

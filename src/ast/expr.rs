use super::Span;

pub struct Expr {
    span: Span,
    kind: ExprKind,
}

pub enum ExprKind {}

use super::{BinOp, Lit, UnaryOp, P};
use crate::lexer::Span;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
crate struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

impl Expr {
    pub fn new(span: Span, kind: ExprKind) -> Self {
        Self { span, kind }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug, PartialEq)]
crate enum ExprKind {
    Lit(Lit),
    Bin(BinOp, P<Expr>, P<Expr>),
    Unary(UnaryOp, P<Expr>),
    Paren(P<Expr>),
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lit(lit) => write!(f, "{}", lit),
            Self::Bin(op, l, r) => write!(f, "({} {} {})", op, l, r),
            Self::Unary(op, expr) => write!(f, "{}{}", op, expr),
            Self::Paren(expr) => write!(f, "({})", expr),
        }
    }
}

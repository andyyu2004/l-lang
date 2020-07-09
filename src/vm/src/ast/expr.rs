use super::{BinOp, Block, Lit, NodeId, Path, UnaryOp, P};
use crate::span::Span;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
crate struct Expr {
    pub span: Span,
    pub id: NodeId,
    pub kind: ExprKind,
}

impl Expr {
    pub fn new(span: Span, id: NodeId, kind: ExprKind) -> Self {
        Self { span, id, kind }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug, PartialEq, Clone)]
crate enum ExprKind {
    Lit(Lit),
    Bin(BinOp, P<Expr>, P<Expr>),
    Unary(UnaryOp, P<Expr>),
    Paren(P<Expr>),
    Block(P<Block>),
    Path(Path),
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lit(lit) => write!(f, "{}", lit),
            Self::Bin(op, l, r) => write!(f, "({} {} {})", op, l, r),
            Self::Unary(op, expr) => write!(f, "{}{}", op, expr),
            Self::Paren(expr) => write!(f, "({})", expr),
            Self::Block(_) => todo!(),
            Self::Path(path) => write!(f, "{}", path),
        }
    }
}

use super::{BinOp, Block, FnSig, Lit, NodeId, Path, UnaryOp, P};
use crate::span::Span;
use crate::util;
use std::fmt::{self, Display, Formatter};

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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    Tuple(Vec<P<Expr>>),
    Lambda(FnSig, P<Expr>),
    Call(P<Expr>, Vec<P<Expr>>),
}

impl Display for ExprKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lit(lit) => write!(fmt, "{}", lit),
            Self::Bin(op, l, r) => write!(fmt, "({} {} {})", op, l, r),
            Self::Unary(op, expr) => write!(fmt, "({}{})", op, expr),
            Self::Paren(expr) => write!(fmt, "({})", expr),
            Self::Block(_) => todo!(),
            Self::Path(path) => write!(fmt, "{}", path),
            Self::Tuple(xs) => write!(fmt, "({})", util::join(xs, ",")),
            Self::Lambda(sig, body) => write!(fmt, "fn ({}) => {}", sig, body),
            Self::Call(f, args) => write!(fmt, "({} {})", f, util::join(args, " ")),
        }
    }
}

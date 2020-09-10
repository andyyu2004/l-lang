use super::{BinOp, Block, Field, FnSig, Ident, Lit, NodeId, Path, UnaryOp, P};
use crate::span::Span;
use crate::util;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub span: Span,
    pub id: NodeId,
    pub kind: ExprKind,
}

impl Expr {
    pub fn is_named_closure(&self) -> bool {
        match self.kind {
            ExprKind::Closure(name, ..) => name.is_some(),
            _ => false,
        }
    }

    pub fn is_diverging(&self) -> bool {
        match self.kind {
            ExprKind::Ret(_) => true,
            ExprKind::Lit(_)
            | ExprKind::Bin(..)
            | ExprKind::Unary(..)
            | ExprKind::Paren(_)
            | ExprKind::Block(_)
            | ExprKind::Path(_)
            | ExprKind::Tuple(_)
            | ExprKind::Assign(..)
            | ExprKind::Closure(..)
            | ExprKind::Call(..)
            | ExprKind::If(..)
            | ExprKind::Field(..)
            | ExprKind::Struct(..) => false,
        }
    }
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
pub enum ExprKind {
    Lit(Lit),
    Bin(BinOp, P<Expr>, P<Expr>),
    Unary(UnaryOp, P<Expr>),
    Paren(P<Expr>),
    Block(P<Block>),
    Path(Path),
    Tuple(Vec<P<Expr>>),
    Ret(Option<P<Expr>>),
    Assign(P<Expr>, P<Expr>),
    Closure(Option<Ident>, FnSig, P<Expr>),
    Call(P<Expr>, Vec<P<Expr>>),
    If(P<Expr>, P<Block>, Option<P<Expr>>),
    Struct(Path, Vec<Field>),
    Field(P<Expr>, Ident),
}

impl Display for ExprKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lit(lit) => write!(fmt, "{}", lit),
            Self::Bin(op, l, r) => write!(fmt, "({} {} {})", op, l, r),
            Self::Unary(op, expr) => write!(fmt, "({}{})", op, expr),
            Self::Paren(expr) => write!(fmt, "({})", expr),
            Self::Assign(l, r) => write!(fmt, "{} = {}", l, r),
            Self::Block(block) => write!(fmt, "{}", block),
            Self::Path(path) => write!(fmt, "{}", path),
            Self::Tuple(xs) => write!(fmt, "({})", util::join(xs, ",")),
            Self::Call(f, args) => write!(fmt, "({} {})", f, util::join(args, " ")),
            Self::Struct(path, fields) => todo!(),
            Self::Field(expr, ident) => write!(fmt, "{}.{}", expr, ident),
            Self::Closure(name, sig, body) => match name {
                Some(name) => write!(fmt, "fn {} ({}) => {}", name, sig, body),
                None => write!(fmt, "fn ({}) => {}", sig, body),
            },
            Self::Ret(expr) => match expr {
                Some(expr) => write!(fmt, "{}", expr),
                None => write!(fmt, ""),
            },
            Self::If(c, l, r) => match r {
                Some(r) => write!(fmt, "if {} {} {}", c, l, r),
                None => write!(fmt, "if {} {}", c, l),
            },
        }
    }
}

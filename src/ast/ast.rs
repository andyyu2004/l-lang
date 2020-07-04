use crate::lexer::{Span, Symbol, Tok, TokenKind};
use std::fmt::Display;

#[derive(Clone, Debug)]
crate struct Ident {
    span: Span,
    symbol: Symbol,
}

#[derive(Clone, Debug)]
crate struct Spanned<T> {
    span: Span,
    node: T,
}

#[derive(Clone, Debug)]
crate struct Path {
    pub span: Span,
    pub segments: Vec<PathSegment>,
}

#[derive(Clone, Debug)]
crate struct PathSegment {
    pub ident: Ident,
    pub args: Option<()>,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
crate enum Lit {
    Int(i64),
    Uint(u64),
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(i) => write!(f, "{}", i),
            Self::Uint(u) => write!(f, "{}", u),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
crate enum BinOp {
    Mul,
    Div,
    Add,
    Sub,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
        }
    }
}

impl From<Tok> for BinOp {
    fn from(t: Tok) -> Self {
        match t.kind {
            TokenKind::Plus => Self::Add,
            TokenKind::Minus => Self::Sub,
            TokenKind::Star => Self::Mul,
            TokenKind::Slash => Self::Div,
            k => panic!("Invalid binary operator `{:?}`", k),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
crate enum UnaryOp {
    Neg,
    Not,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Neg => write!(f, "-"),
            Self::Not => write!(f, "!"),
        }
    }
}

impl From<Tok> for UnaryOp {
    fn from(t: Tok) -> Self {
        match t.kind {
            TokenKind::Minus => Self::Neg,
            TokenKind::Not => Self::Not,
            k => panic!("Invalid unary operator `{:?}`", k),
        }
    }
}

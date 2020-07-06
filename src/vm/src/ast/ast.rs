use crate::lexer::{Symbol, Tok, TokenType};
use crate::span::Span;
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

#[derive(Debug, PartialEq, Copy, Clone)]
crate enum Lit {
    Num(f64),
    Bool(bool),
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Num(i) => write!(f, "{}", i),
            Self::Bool(b) => write!(f, "{}", b),
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
        match t.ttype {
            TokenType::Plus => Self::Add,
            TokenType::Minus => Self::Sub,
            TokenType::Star => Self::Mul,
            TokenType::Slash => Self::Div,
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
        match t.ttype {
            TokenType::Minus => Self::Neg,
            TokenType::Not => Self::Not,
            k => panic!("Invalid unary operator `{:?}`", k),
        }
    }
}

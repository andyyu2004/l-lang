use crate::{
    error::ParseResult, lexer::{Tok, TokenKind}, parser::{Parse, Parser}
};
use std::fmt::Display;

#[derive(Debug, PartialEq)]
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

impl Parse for Lit {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
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

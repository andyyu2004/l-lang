use crate::{LiteralKind, Token};

pub struct TokenStream {}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenTree {
    Group(TokenTreeGroup),
    Token(Token),
    Literal(LiteralKind),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTreeGroup {}

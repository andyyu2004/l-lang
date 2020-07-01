use super::Parse;
use crate::{ast::*, error::ParseResult};

#[derive(Copy, Clone)]
pub struct Token;
pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
}

impl Iterator for Parser {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.tokens.len() - 1 {
            None
        } else {
            Some(self.tokens[self.i])
        }
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, i: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Prog> {
        todo!()
    }

    pub fn parse_item(&mut self) -> ParseResult<Item> {
        todo!()
    }

    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        Expr::parse(self)
    }
}

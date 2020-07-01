use crate::{ast::*, error::ParseResult};
use regexlexer::Token;

pub struct Parser<'src> {
    tokens: Vec<Token<'src>>,
    i: usize,
}

impl<'src> Iterator for Parser<'src> {
    type Item = Token<'src>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.tokens.len() - 1 {
            None
        } else {
            Some(self.tokens[self.i])
        }
    }
}

impl<'src> Parser<'src> {
    pub fn new(tokens: Vec<Token<'src>>) -> Self {
        Self { tokens, i: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Prog> {
        todo!()
    }

    pub fn parse_item(&mut self) -> ParseResult<Item> {
        todo!()
    }

    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        todo!()
    }
}

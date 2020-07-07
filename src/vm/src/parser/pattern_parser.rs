use super::{Parse, Parser, PunctuatedParser, VisibilityParser};
use crate::ast::{FnSig, Generics, Item, ItemKind, Param, Pattern};
use crate::error::ParseResult;
use crate::lexer::{Tok, TokenType};

pub struct PatternParser;

impl Parse for PatternParser {
    type Output = Pattern;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        todo!()
    }
}

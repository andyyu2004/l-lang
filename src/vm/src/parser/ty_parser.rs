use super::{Parse, Parser, PunctuatedParser, VisibilityParser};
use crate::ast::{FnSig, Generics, Item, ItemKind, Param, Ty, P};
use crate::error::ParseResult;
use crate::lexer::{Tok, TokenType};

crate struct TyParser;

impl Parse for TyParser {
    type Output = P<Ty>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        todo!()
    }
}

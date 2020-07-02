use crate::parser::{Parse, Parser};
use crate::{error::ParseResult, lexer::Span};

#[derive(Debug)]
crate struct Item {
    pub span: Span,
    pub kind: ItemKind,
}

impl Parse for Item {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
crate enum ItemKind {}

impl Parse for ItemKind {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        todo!()
    }
}

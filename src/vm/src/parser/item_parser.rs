use super::{Parse, Parser};
use crate::{ast::Item, error::ParseResult};

crate struct ItemParser;

impl Parse for ItemParser {
    type Output = Item;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        todo!()
    }
}

use super::Parser;
use crate::error::ParseResult;

crate trait Parse: Sized {
    type Output;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output>;
}

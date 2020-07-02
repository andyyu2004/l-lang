use super::Parser;
use crate::error::ParseResult;

crate trait Parse: Sized {
    fn parse(parser: &mut Parser) -> ParseResult<Self>;
}

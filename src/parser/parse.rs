use super::Parser;
use crate::error::ParseResult;

pub trait Parse: Sized {
    fn parse(parser: &mut Parser) -> ParseResult<Self>;
}

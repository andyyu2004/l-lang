use super::Parser;
use crate::error::ParseResult;

crate trait Parse: Sized {
    type Output;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output>;
}

// implement Parser for all `parser-like` functions
impl<F, R> Parse for F
where
    F: FnMut(&mut Parser) -> ParseResult<R>,
{
    type Output = R;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        self(parser)
    }
}

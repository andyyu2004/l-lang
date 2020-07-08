use super::Parser;
use crate::error::ParseResult;

crate trait Parse: Sized {
    type Output;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output>;

    fn or<P>(self, other: P) -> OrParser<Self, P>
    where
        P: Parse<Output = Self::Output>,
    {
        OrParser { fst: self, snd: other }
    }
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

crate struct OrParser<P, Q> {
    fst: P,
    snd: Q,
}

impl<P, Q, R> Parse for OrParser<P, Q>
where
    P: Parse<Output = R>,
    Q: Parse<Output = R>,
{
    type Output = R;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        match parser.try_parse(&mut self.fst) {
            Some(p) => Ok(p),
            None => self.snd.parse(parser),
        }
    }
}

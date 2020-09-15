use super::Parser;
use crate::error::ParseResult;
use crate::span::Span;

pub trait Parse<'a>: Sized {
    type Output;
    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output>;

    fn or<P>(self, other: P) -> OrParser<Self, P>
    where
        P: Parse<'a, Output = Self::Output>,
    {
        OrParser { fst: self, snd: other }
    }

    fn spanned(self, include_prev: bool) -> SpannedParser<Self> {
        SpannedParser { inner: self, include_prev }
    }
}

// implement Parser for all `parser-like` functions
impl<'a, F, R> Parse<'a> for F
where
    F: FnMut(&mut Parser<'a>) -> ParseResult<'a, R>,
{
    type Output = R;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        self(parser)
    }
}

pub struct SpannedParser<P> {
    inner: P,
    include_prev: bool,
}

impl<'a, P: Parse<'a>> Parse<'a> for SpannedParser<P> {
    type Output = (Span, P::Output);

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        parser.with_span(&mut self.inner, self.include_prev)
    }
}

pub struct OrParser<P, Q> {
    fst: P,
    snd: Q,
}

impl<'a, P, Q, R> Parse<'a> for OrParser<P, Q>
where
    P: Parse<'a, Output = R>,
    Q: Parse<'a, Output = R>,
{
    type Output = R;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        match parser.try_parse(&mut self.fst) {
            Some(p) => Ok(p),
            None => self.snd.parse(parser),
        }
    }
}

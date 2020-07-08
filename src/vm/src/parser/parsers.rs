//! general use parsers

use super::{Parse, Parser};
use crate::ast::{Ident, Path, PathSegment, Visibility, VisibilityKind};
use crate::error::ParseResult;
use crate::{
    lexer::{Tok, TokenType}, span::Span
};

crate struct VisibilityParser;

impl Parse for VisibilityParser {
    type Output = Visibility;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(pub_keyword) = parser.accept(TokenType::Pub) {
            Ok(Visibility { span: pub_keyword.span, node: VisibilityKind::Public })
        } else {
            Ok(Visibility { span: parser.empty_span(), node: VisibilityKind::Private })
        }
    }
}

/// implement Parser for TokenType to be used as a separator
impl Parse for TokenType {
    type Output = Tok;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        parser.expect(*self)
    }
}

/// parses a given parser zero or more times punctuated by some given separator parser
/// this parser accepts a trailing separator
///
/// <punctuated> = Ɛ | <inner> ( <sep> <inner> )* <sep>?
crate struct PunctuatedParser<P, S> {
    pub inner: P,
    pub separator: S,
}

impl<P, S> Parse for PunctuatedParser<P, S>
where
    P: Parse,
    S: Parse,
{
    type Output = Vec<P::Output>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut vec = vec![];
        // if the first parse already fails then just return empty vector
        let p = match self.inner.parse(parser) {
            Ok(p) => p,
            Err(_) => return Ok(vec),
        };
        vec.push(p);
        while self.separator.parse(parser).is_ok() {
            vec.push(self.inner.parse(parser)?);
        }
        // parse the trailing separator if there is one
        let _ = self.separator.parse(parser);
        Ok(vec)
    }
}

/// similar to `PunctuatedParser` except parses one or more occurences of `inner`
/// accepts trailing separator
crate struct Punctuated1Parser<P, S> {
    pub inner: P,
    pub separator: S,
}

impl<P, S> Parse for Punctuated1Parser<P, S>
where
    P: Parse,
    S: Parse,
{
    type Output = Vec<P::Output>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut vec = vec![self.inner.parse(parser)?];
        while self.separator.parse(parser).is_ok() {
            vec.push(self.inner.parse(parser)?);
        }
        let _ = self.separator.parse(parser);
        Ok(vec)
    }
}

/// similar to `PunctuatedParser` except a single element tuple must have a trailing comma (to
/// differentiate it from a parenthesization)
/// <tuple> = () | '(' ( <inner> , )+ <inner>? ')'
crate struct TupleParser<P> {
    pub inner: P,
    pub open_paren: Tok,
}

impl<P> Parse for TupleParser<P>
where
    P: Parse,
{
    type Output = Vec<P::Output>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut vec = vec![];
        if parser.accept(TokenType::CloseParen).is_some() {
            return Ok(vec);
        }
        while parser.accept(TokenType::CloseParen).is_none() {
            vec.push(self.inner.parse(parser)?);
            if parser.accept(TokenType::Comma).is_none() {
                parser.expect(TokenType::CloseParen)?;
            }
        }
        Ok(vec)
    }
}

/// parser some inner parser within parentheses
crate struct ParenParser<P> {
    pub open_paren: Tok,
    pub inner: P,
}

impl<P> Parse for ParenParser<P>
where
    P: Parse,
{
    type Output = (P::Output, Span);
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let p = self.inner.parse(parser)?;
        let close_paren = parser.expect(TokenType::CloseParen)?;
        let span = self.open_paren.span.merge(&close_paren.span);
        Ok((p, span))
    }
}

crate struct PathParser;

impl Parse for PathParser {
    type Output = Path;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let separator = |parser: &mut Parser| {
            parser.expect(TokenType::Colon)?;
            parser.expect(TokenType::Colon)
        };
        let (segments, span) =
            parser.with_span(&mut Punctuated1Parser { inner: PathSegmentParser, separator })?;
        Ok(Path { span, segments })
    }
}

crate struct PathSegmentParser;

impl Parse for PathSegmentParser {
    type Output = PathSegment;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let ident = parser.expect_ident()?;
        // with the generics of the initial ident
        Ok(PathSegment { ident, id: parser.mk_id(), args: None })
    }
}

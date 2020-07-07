//! general use parsers

use super::{Parse, Parser};
use crate::ast::{Visibility, VisibilityKind};
use crate::error::ParseResult;
use crate::lexer::{Tok, TokenType};

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
/// this parser accepts trailing seperators
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

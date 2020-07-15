use super::{parsers::*, Parse, Parser};
use crate::ast::{Pattern, PatternKind, P};
use crate::error::{ParseError, ParseResult};
use crate::lexer::TokenType;

pub struct PatParser;

impl Parse for PatParser {
    type Output = P<Pattern>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(token) = parser.accept(TokenType::Underscore) {
            Ok(parser.mk_pat(token.span, PatternKind::Wildcard))
        } else if let Some(ident) = parser.accept_ident() {
            Ok(parser.mk_pat(ident.span, PatternKind::Ident(ident, None)))
        } else if let Some(open_paren) = parser.accept(TokenType::OpenParen) {
            if let Some((span, pattern)) =
                parser.try_parse(&mut ParenParser { inner: PatParser }.spanned(true))
            {
                Ok(parser.mk_pat(span, PatternKind::Paren(pattern)))
            } else {
                let (span, patterns) =
                    TupleParser { inner: PatParser }.spanned(true).parse(parser)?;
                Ok(parser.mk_pat(span, PatternKind::Tuple(patterns)))
            }
        } else {
            Err(ParseError::unimpl())
        }
    }
}

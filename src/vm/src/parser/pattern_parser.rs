use super::parsers::*;
use super::{Parse, Parser};
use crate::ast::{Mutability, Pattern, PatternKind, P};
use crate::error::{ParseError, ParseResult};
use crate::lexer::TokenType;
use crate::span::Span;

pub struct PatParser;

impl<'a> Parse<'a> for PatParser {
    type Output = P<Pattern>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        if let Some(token) = parser.accept(TokenType::Underscore) {
            Ok(parser.mk_pat(token.span, PatternKind::Wildcard))
        } else if parser.on_ident()? {
            let path = parser.parse_path()?;
            if path.segments.len() == 1 {
                let ident = path.segments[0].ident;
                return Ok(
                    parser.mk_pat(ident.span, PatternKind::Ident(ident, None, Mutability::Imm))
                );
            }
            if parser.accept(TokenType::OpenBrace).is_some() {
                todo!()
            } else if parser.accept(TokenType::OpenParen).is_some() {
                let (span, patterns) = parser.parse_tuple_pat()?;
                Ok(parser.mk_pat(path.span.merge(span), PatternKind::Variant(path, patterns)))
            } else {
                Ok(parser.mk_pat(path.span, PatternKind::Path(path)))
            }
        } else if let Some(m) = parser.accept(TokenType::Mut) {
            let ident = parser.expect_ident()?;
            let pat = PatternKind::Ident(ident, None, Mutability::Mut);
            Ok(parser.mk_pat(m.span.merge(ident.span), pat))
        } else if let Some(open_paren) = parser.accept(TokenType::OpenParen) {
            if let Some((span, pattern)) =
                parser.try_parse(&mut ParenParser { inner: PatParser }.spanned(true))
            {
                Ok(parser.mk_pat(span, PatternKind::Paren(pattern)))
            } else {
                let (span, patterns) = parser.parse_tuple_pat()?;
                Ok(parser.mk_pat(span, PatternKind::Tuple(patterns)))
            }
        } else {
            // otherwise try parse a path
            Err(parser.err(parser.empty_span(), ParseError::Unimpl))
        }
    }
}

impl<'a> Parser<'a> {
    fn parse_tuple_pat(&mut self) -> ParseResult<'a, (Span, Vec<P<Pattern>>)> {
        TupleParser { inner: PatParser }.spanned(true).parse(self)
    }
}

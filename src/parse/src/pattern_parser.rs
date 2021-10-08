use super::*;
use crate::{ParseError, ParseResult};
use ast::{Mutability, Pattern, PatternKind, P};
use lex::TokenKind;
use span::Span;

pub struct PatParser;

impl<'a> Parse<'a> for PatParser {
    type Output = P<Pattern>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        if let Some(token) = parser.accept(TokenKind::Underscore) {
            Ok(parser.mk_pat(token.span, PatternKind::Wildcard))
        } else if parser.is_ident()?.is_some() {
            let path = parser.parse_expr_path()?;
            if parser.accept(TokenKind::OpenBrace).is_some() {
                let (span, fields) =
                    PunctuatedParser { inner: FieldPatParser, separator: TokenKind::Comma }
                        .spanned(false)
                        .parse(parser)?;
                parser.expect(TokenKind::CloseBrace)?;
                let span = path.span.merge(span);
                Ok(parser.mk_pat(span, PatternKind::Struct(path, fields)))
            } else if parser.accept(TokenKind::OpenParen).is_some() {
                let (span, patterns) = parser.parse_tuple_pat()?;
                Ok(parser.mk_pat(path.span.merge(span), PatternKind::Variant(path, patterns)))
            } else if path.segments.len() == 1 {
                let ident = path.segments[0].ident;
                Ok(parser.mk_pat(ident.span, PatternKind::Ident(ident, None, Mutability::Imm)))
            } else {
                Ok(parser.mk_pat(path.span, PatternKind::Path(path)))
            }
        } else if let Some(amp) = parser.accept(TokenKind::And) {
            let pat = parser.parse_pattern()?;
            Ok(parser.mk_pat(amp.span.merge(pat.span), PatternKind::Box(pat)))
        } else if let Some(m) = parser.accept(TokenKind::Mut) {
            let ident = parser.expect_lident()?;
            let pat = PatternKind::Ident(ident, None, Mutability::Mut);
            Ok(parser.mk_pat(m.span.merge(ident.span), pat))
        } else if let Some(_open_paren) = parser.accept(TokenKind::OpenParen) {
            if let Some((span, pattern)) =
                parser.try_parse(&mut ParenParser { inner: PatParser }.spanned(true))
            {
                Ok(parser.mk_pat(span, PatternKind::Paren(pattern)))
            } else {
                let (span, patterns) = parser.parse_tuple_pat()?;
                Ok(parser.mk_pat(span, PatternKind::Tuple(patterns)))
            }
        } else if let Some((kind, span)) = parser.accept_literal() {
            let expr = LiteralParser { kind, span }.parse(parser)?;
            Ok(parser.mk_pat(span, PatternKind::Lit(expr)))
        } else if let Some(false_kw) = parser.accept(TokenKind::False) {
            let expr = parser.mk_expr(false_kw.span, ExprKind::Lit(Lit::Bool(false)));
            Ok(parser.mk_pat(false_kw.span, PatternKind::Lit(expr)))
        } else if let Some(true_kw) = parser.accept(TokenKind::True) {
            let expr = parser.mk_expr(true_kw.span, ExprKind::Lit(Lit::Bool(true)));
            Ok(parser.mk_pat(true_kw.span, PatternKind::Lit(expr)))
        } else {
            Err(parser.build_err(parser.empty_span(), ParseError::Unimpl))
        }
    }
}

struct FieldPatParser;

impl<'a> Parse<'a> for FieldPatParser {
    type Output = FieldPat;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let ident = parser.expect_ident()?;
        let pat = if parser.accept(TokenKind::Colon).is_some() {
            parser.parse_pattern()?
        } else {
            // struct shorthand
            // let S { x } = s <=> let S { x: x } = s;
            parser.mk_pat(ident.span, PatternKind::Ident(ident, None, Mutability::Imm))
        };
        let span = ident.span.merge(pat.span);
        Ok(FieldPat { ident, pat, span })
    }
}

impl<'a> Parser<'a> {
    fn parse_tuple_pat(&mut self) -> ParseResult<'a, (Span, Vec<P<Pattern>>)> {
        TupleParser { inner: PatParser }.spanned(true).parse(self)
    }
}

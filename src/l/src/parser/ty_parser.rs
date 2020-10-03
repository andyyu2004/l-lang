use super::parsers::*;
use super::*;
use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::lexer::{symbol, Tok, TokenType};

#[derive(Copy, Clone)]
pub struct TyParser {
    pub allow_infer: bool,
}

impl<'a> Parse<'a> for TyParser {
    type Output = P<Ty>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        if let Some(fn_kw) = parser.accept(TokenType::Fn) {
            let (span, (inputs, output)) = FnTyParser.spanned(true).parse(parser)?;
            Ok(parser.mk_ty(span, TyKind::Fn(inputs, output)))
        } else if let Some(uscore) = parser.accept(TokenType::Underscore) {
            if !self.allow_infer {
                parser.err(uscore.span, ParseError::ElidedTypeNotAllowedInThisContext).emit()
            }
            Ok(parser.mk_ty(uscore.span, TyKind::Infer))
        } else if let Some(lparen) = parser.accept(TokenType::OpenParen) {
            let mut paren_parser = ParenParser { inner: *self }.spanned(true);
            if let Some((span, ty)) = parser.try_parse(&mut paren_parser) {
                Ok(parser.mk_ty(span, TyKind::Paren(ty)))
            } else {
                let mut tuple_parser = TupleParser { inner: *self }.spanned(true);
                let (span, tys) = tuple_parser.parse(parser)?;
                Ok(parser.mk_ty(span, TyKind::Tuple(tys)))
            }
        } else if let Some(amp) = parser.accept(TokenType::And) {
            let m = parser.parse_mutability();
            let ty = self.parse(parser)?;
            Ok(parser.mk_ty(amp.span.merge(ty.span), TyKind::Ptr(m, ty)))
        } else if let Some(lsq) = parser.accept(TokenType::OpenSqBracket) {
            let ty = self.parse(parser)?;
            let rsq = parser.expect(TokenType::CloseSqBracket)?;
            Ok(parser.mk_ty(lsq.span.merge(rsq.span), TyKind::Array(ty)))
        } else if parser.ident()?.is_some() {
            let path = parser.parse_type_path()?;
            Ok(parser.mk_ty(path.span, TyKind::Path(path)))
        } else {
            Err(parser.err(parser.empty_span(), ParseError::Unimpl))
        }
    }
}

/// fn (<ty>...) (-> <ty>)?
pub struct FnTyParser;

impl<'a> Parse<'a> for FnTyParser {
    type Output = (Vec<P<Ty>>, Option<P<Ty>>);

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        parser.expect(TokenType::OpenParen)?;
        let inputs = TupleParser { inner: TyParser { allow_infer: false } }.parse(parser)?;
        let output =
            parser.accept(TokenType::RArrow).map(|_| parser.parse_ty(false)).transpose()?;
        Ok((inputs, output))
    }
}
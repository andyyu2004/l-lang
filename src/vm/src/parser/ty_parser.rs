use super::parsers::*;
use super::*;
use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::lexer::{Tok, TokenType};

crate struct TyParser;

impl Parse for TyParser {
    type Output = P<Ty>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(fn_kw) = parser.accept(TokenType::Fn) {
            let (span, (inputs, output)) = FnTyParser.spanned(true).parse(parser)?;
            Ok(parser.mk_ty(span, TyKind::Fn(inputs, output)))
        } else if let Some(lparen) = parser.accept(TokenType::OpenParen) {
            let mut paren_parser = ParenParser { inner: Self }.spanned(true);
            if let Some((span, ty)) = parser.try_parse(&mut paren_parser) {
                Ok(parser.mk_ty(span, TyKind::Paren(ty)))
            } else {
                let mut tuple_parser = TupleParser { inner: Self }.spanned(true);
                let (span, tys) = tuple_parser.parse(parser)?;
                Ok(parser.mk_ty(span, TyKind::Tuple(tys)))
            }
        } else if let Some(lsq) = parser.accept(TokenType::OpenSqBracket) {
            let ty = self.parse(parser)?;
            let rsq = parser.expect(TokenType::CloseSqBracket)?;
            Ok(parser.mk_ty(lsq.span.merge(rsq.span), TyKind::Array(ty)))
        } else if let TokenType::Ident(_) = parser.safe_peek()?.ttype {
            let path = PathParser.parse(parser)?;
            Ok(parser.mk_ty(path.span, TyKind::Path(path)))
        } else {
            Err(ParseError::unimpl())
        }
    }
}

/// fn (<ty>...) (-> <ty>)?
crate struct FnTyParser;

impl Parse for FnTyParser {
    type Output = (Vec<P<Ty>>, Option<P<Ty>>);

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        parser.expect(TokenType::OpenParen)?;
        let inputs = TupleParser { inner: TyParser }.parse(parser)?;
        let output =
            parser.accept(TokenType::RArrow).map(|_| TyParser.parse(parser)).transpose()?;
        Ok((inputs, output))
    }
}

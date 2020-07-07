use super::*;
use crate::ast::{FnSig, Generics, Item, ItemKind, Param};
use crate::error::ParseResult;
use crate::lexer::{Tok, TokenType};

const ITEM_KEYWORDS: [TokenType; 3] = [TokenType::Fn, TokenType::Struct, TokenType::Enum];

crate struct ItemParser;

impl Parse for ItemParser {
    type Output = Item;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let vis = VisibilityParser.parse(parser)?;
        let item_kind_keyword = parser.expect_one_of(&ITEM_KEYWORDS)?;
        let ident = parser.expect_ident()?;
        let (kind, kind_span) =
            parser.with_span(|parser: &mut Parser| match item_kind_keyword.ttype {
                TokenType::Fn => FnParser { fn_keyword: item_kind_keyword }.parse(parser),
                TokenType::Struct => todo!(),
                TokenType::Enum => todo!(),
                _ => unreachable!(),
            })?;

        Ok(Item { span: vis.span.merge(&kind_span), vis, ident, kind })
    }
}

crate struct FnParser {
    fn_keyword: Tok,
}

impl Parse for FnParser {
    type Output = ItemKind;
    /// assumes that { <vis> fn <ident> } has already been parsed
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let generics = Generics { span: parser.empty_span() };
        let sig = FnSigParser.parse(parser)?;
        let block = if parser.accept(TokenType::Semi).is_some() { None } else { None };
        Ok(ItemKind::Fn(sig, generics, block))
    }
}

crate struct FnSigParser;

impl Parse for FnSigParser {
    type Output = FnSig;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        parser.expect(TokenType::OpenParen)?;
        PunctuatedParser { inner: ParamParser, separator: TokenType::Comma };
        parser.expect(TokenType::CloseParen)?;
        todo!()
    }
}

crate struct ParamParser;

impl Parse for ParamParser {
    type Output = Param;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let pattern = PatternParser.parse(parser)?;
        let ty = TyParser.parse(parser)?;
        Ok(Param { span: pattern.span.merge(&ty.span), pattern, ty })
    }
}

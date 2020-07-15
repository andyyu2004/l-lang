use super::*;
use crate::ast::{FnSig, Generics, Item, ItemKind, Param, P};
use crate::error::ParseResult;
use crate::lexer::{Tok, TokenType};

const ITEM_KEYWORDS: [TokenType; 3] = [TokenType::Fn, TokenType::Struct, TokenType::Enum];

crate struct ItemParser;

impl Parse for ItemParser {
    type Output = P<Item>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let vis = VisibilityParser.parse(parser)?;
        let item_kind_keyword = parser.expect_one_of(&ITEM_KEYWORDS)?;
        let ident = parser.expect_ident()?;
        let (kind_span, kind) = parser.with_span(
            &mut |parser: &mut Parser| match item_kind_keyword.ttype {
                TokenType::Fn => FnParser { fn_kw: item_kind_keyword }.parse(parser),
                TokenType::Struct => todo!(),
                TokenType::Enum => todo!(),
                _ => unreachable!(),
            },
            false,
        )?;

        Ok(parser.mk_item(vis.span.merge(&kind_span), vis, ident, kind))
    }
}

crate struct FnParser {
    fn_kw: Tok,
}

impl Parse for FnParser {
    type Output = ItemKind;

    /// assumes that { <vis> fn <ident> } has already been parsed
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let generics = Generics { id: parser.mk_id(), span: parser.empty_span() };
        let sig = FnSigParser.parse(parser)?;
        let block = if let Some(open_brace) = parser.accept(TokenType::OpenBrace) {
            Some(BlockParser { open_brace }.parse(parser)?)
        } else {
            parser.expect(TokenType::Semi)?;
            None
        };
        Ok(ItemKind::Fn(sig, generics, block))
    }
}

crate struct FnSigParser;

impl Parse for FnSigParser {
    type Output = FnSig;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        parser.expect(TokenType::OpenParen)?;
        let mut param_parser = PunctuatedParser { inner: ParamParser, separator: TokenType::Comma };
        let inputs = param_parser.parse(parser)?;
        parser.expect(TokenType::CloseParen)?;
        Ok(FnSig { inputs, output: None })
    }
}

crate struct ParamParser;

impl Parse for ParamParser {
    type Output = Param;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let pattern = PatParser.parse(parser)?;
        let ty = TyParser.parse(parser)?;
        Ok(Param { span: pattern.span.merge(&ty.span), id: parser.mk_id(), pattern, ty })
    }
}

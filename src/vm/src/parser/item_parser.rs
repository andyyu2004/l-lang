use super::*;
use crate::ast::{Expr, ExprKind, FnSig, Generics, Item, ItemKind, Param, P};
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
        let generics = GenericsParser.parse(parser)?;
        let sig = FnSigParser { require_type_annotations: true }.parse(parser)?;
        let block = if let Some(open_brace) = parser.accept(TokenType::OpenBrace) {
            Some(BlockParser { open_brace }.parse(parser)?)
        } else {
            parser.expect(TokenType::Semi)?;
            None
        };
        let expr = block.map(|block| parser.mk_expr(block.span, ExprKind::Block(block)));
        Ok(ItemKind::Fn(sig, generics, expr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{span::Span, Driver};
    use indexed_vec::Idx;

    macro parse($src:expr) {{
        let driver = Driver::new($src);
        driver.parse().unwrap()
    }}

    macro fmt($src:expr) {{
        let prog = parse!($src);
        format!("{}", prog)
    }}

    #[test]
    fn parse_generics() {
        let _prog = parse!("fn test<T, U>() -> bool { false }");
    }
}

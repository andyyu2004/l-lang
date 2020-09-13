use super::*;
use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::lexer::{Tok, TokenType};

const ITEM_KEYWORDS: [TokenType; 3] = [TokenType::Fn, TokenType::Struct, TokenType::Enum];

pub struct ItemParser;

impl Parse for ItemParser {
    type Output = P<Item>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let vis = VisibilityParser.parse(parser)?;
        let kw = parser.expect_one_of(&ITEM_KEYWORDS)?;
        let ident = parser.expect_ident()?;
        let (kind_span, kind) = parser.with_span(
            &mut |parser: &mut Parser| match kw.ttype {
                TokenType::Fn => FnParser { fn_kw: kw }.parse(parser),
                TokenType::Struct => StructDeclParser { struct_kw: kw }.parse(parser),
                TokenType::Enum => EnumParser { enum_kw: kw }.parse(parser),
                TokenType::Type => TypeAliasParser { type_kw: kw }.parse(parser),
                _ => unreachable!(),
            },
            false,
        )?;

        Ok(parser.mk_item(vis.span.merge(kind_span), vis, ident, kind))
    }
}

pub struct TypeAliasParser {
    type_kw: Tok,
}

impl Parse for TypeAliasParser {
    type Output = ItemKind;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        todo!()
    }
}

enum FieldForm {
    Struct,
    Tuple,
}

pub struct FieldDeclParser {
    form: FieldForm,
}

impl Parse for FieldDeclParser {
    type Output = FieldDecl;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let vis = VisibilityParser.parse(parser)?;
        let ident = match self.form {
            FieldForm::Struct => {
                let ident = parser.expect_ident()?;
                parser.expect(TokenType::Colon)?;
                Some(ident)
            }
            FieldForm::Tuple => None,
        };
        let ty = TyParser.parse(parser)?;
        let span = vis.span.merge(ty.span);
        Ok(FieldDecl { id: parser.mk_id(), span, vis, ident, ty })
    }
}

pub struct VariantKindParser;

impl Parse for VariantKindParser {
    type Output = VariantKind;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if parser.accept(TokenType::Semi).is_some() {
            Ok(VariantKind::Unit)
        } else if parser.accept(TokenType::OpenParen).is_some() {
            let form = FieldForm::Tuple;
            let fields = TupleParser { inner: FieldDeclParser { form } }.parse(parser)?;
            Ok(VariantKind::Tuple(fields))
        } else if parser.accept(TokenType::OpenBrace).is_some() {
            let fields = PunctuatedParser {
                inner: FieldDeclParser { form: FieldForm::Struct },
                separator: TokenType::Comma,
            }
            .parse(parser)?;
            parser.expect(TokenType::CloseBrace)?;
            Ok(VariantKind::Struct(fields))
        } else {
            Err(ParseError::unimpl())
        }
    }
}

pub struct StructDeclParser {
    struct_kw: Tok,
}

impl Parse for StructDeclParser {
    type Output = ItemKind;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let generics = GenericsParser.parse(parser)?;
        let kind = VariantKindParser.parse(parser)?;
        Ok(ItemKind::Struct(generics, kind))
    }
}

pub struct EnumParser {
    enum_kw: Tok,
}

impl Parse for EnumParser {
    type Output = ItemKind;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        todo!()
    }
}

pub struct FnParser {
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

    #[test]
    fn parse_struct() {
        let _prog = parse!("struct S { x: int }");
        let _prog = parse!("struct S { x: int, y: bool }");
    }

    #[test]
    fn parse_tuple_struct() {
        let _prog = parse!("struct S(number)");
        let _prog = parse!("struct S(number, bool)");
    }
}

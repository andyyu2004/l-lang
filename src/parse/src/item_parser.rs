use crate::*;
use ast::*;
use lex::TokenType;
use span::{with_source_map, ModuleKind};
use std::convert::TryFrom;

const ITEM_KEYWORDS: [TokenType; 9] = [
    TokenType::Fn,
    TokenType::Struct,
    TokenType::Enum,
    TokenType::Const,
    TokenType::Impl,
    TokenType::Extern,
    TokenType::Type,
    TokenType::Use,
    TokenType::Mod,
];

pub struct ItemParser;

impl<'a> Parse<'a> for ItemParser {
    type Output = P<Item>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let vis = VisibilityParser.parse(parser)?;
        // these items have a different syntax to the rest
        if let Some(_impl_kw) = parser.accept(TokenType::Impl) {
            return ImplParser { vis }.parse(parser);
        } else if let Some(_extern_kw) = parser.accept(TokenType::Extern) {
            return ExternParser { vis }.parse(parser);
        } else if let Some(_use_kw) = parser.accept(TokenType::Use) {
            return UseParser { vis }.parse(parser);
        }
        if let Some(_) = parser.accept(TokenType::Mod) {
            let name = parser.expect_lident()?;
            parser.expect(TokenType::Semi)?;
            let module = ModuleParser { name }.parse(parser)?;
            return Ok(parser.mk_item(
                vis.span.merge(name.span),
                vis,
                Ident::empty(),
                ItemKind::Mod(module),
            ));
        }

        let kw = parser.expect_one_of(&ITEM_KEYWORDS)?;
        let ident = parser.expect_ident()?;
        let (kind_span, kind) = parser.with_span(
            &mut |parser: &mut Parser<'a>| match kw.ttype {
                TokenType::Fn => FnParser.parse(parser),
                TokenType::Struct => StructDeclParser.parse(parser),
                TokenType::Enum => EnumParser.parse(parser),
                TokenType::Type => TypeAliasParser.parse(parser),
                _ => unreachable!(),
            },
            false,
        )?;

        parser.accept(TokenType::Semi);

        Ok(parser.mk_item(vis.span.merge(kind_span), vis, ident, kind))
    }
}

pub struct ModuleParser {
    crate name: Ident,
}

impl<'a> Parse<'a> for ModuleParser {
    type Output = Module;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        if with_source_map(|map| map.get(parser.file).file.kind == ModuleKind::File) {
            return Err(
                parser.build_err(self.name.span, ParseError::FileModuleWithSubmodules(self.name))
            );
        }

        let module_file = match with_source_map(|map| map.add_module(parser.file, *self.name)) {
            Some(file) => file,
            None =>
                return Err(parser.build_err(
                    self.name.span,
                    ParseError::UnresolvedModule(
                        with_source_map(|map| map.dir_of(parser.file).to_path_buf()),
                        self.name,
                    ),
                )),
        };

        with_source_map(|map| {
            debug!("found module `{}` at `{}`", self.name, map.path_of(module_file).display())
        });

        parser.with_file(module_file, |parser| {
            let mut items = vec![];
            while !parser.reached_eof() {
                items.push(ItemParser.parse(parser)?);
            }

            let span = if let Some(fst) = items.first() {
                let last = items.last().unwrap();
                fst.span.merge(last.span)
            } else {
                Span::default()
            };

            Ok(Module { name: self.name, span, items })
        })
    }
}

pub struct UseParser {
    vis: Visibility,
}

impl<'a> Parse<'a> for UseParser {
    type Output = P<Item>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let path = parser.parse_module_path()?;
        let span = self.vis.span.merge(path.span);
        let kind = ItemKind::Use(path);
        parser.expect(TokenType::Semi)?;
        Ok(parser.mk_item(span, self.vis, Ident::empty(), kind))
    }
}

pub struct ExternParser {
    vis: Visibility,
}

impl<'a> Parse<'a> for ExternParser {
    type Output = P<Item>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        parser.expect(TokenType::OpenBrace)?;
        let mut foreign_items = vec![];
        let close_brace = loop {
            if let Some(close_brace) = parser.accept(TokenType::CloseBrace) {
                break close_brace;
            }
            let box Item { span, id, kind, vis, ident } = parser.parse_item()?;
            match ForeignItemKind::try_from(kind) {
                Ok(kind) => foreign_items.push(box Item { span, id, vis, ident, kind }),
                Err(kind) => parser.build_err(span, ParseError::InvalidForeignItem(kind)).emit(),
            };
        };

        let span = self.vis.span.merge(close_brace.span);

        let kind = ItemKind::Extern(foreign_items);
        Ok(parser.mk_item(span, self.vis, Ident::empty(), kind))
    }
}

pub struct ImplParser {
    vis: Visibility,
}

impl<'a> Parse<'a> for ImplParser {
    type Output = P<Item>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let generics = parser.parse_generics()?;
        let mut trait_path = Some(parser.parse_type_path()?);
        let self_ty = if parser.accept(TokenType::For).is_some() {
            parser.parse_ty(false)
        } else {
            // reinterpret the trait path as the self type
            let ty_path = trait_path.take().unwrap();
            parser.mk_ty(ty_path.span, TyKind::Path(ty_path))
        };
        parser.expect(TokenType::OpenBrace)?;
        let mut items = vec![];
        let close_brace = loop {
            if let Some(close_brace) = parser.accept(TokenType::CloseBrace) {
                break close_brace;
            }
            let box Item { span, id, kind, vis, ident } = parser.parse_item()?;
            match AssocItemKind::try_from(kind) {
                Ok(kind) => items.push(box Item { span, id, vis, ident, kind }),
                Err(kind) => parser.build_err(span, ParseError::InvalidImplItem(kind)).emit(),
            };
        };
        let span = self.vis.span.merge(close_brace.span);
        let kind = ItemKind::Impl { generics, trait_path, self_ty, items };
        Ok(parser.mk_item(span, self.vis, Ident::empty(), kind))
    }
}

pub struct TypeAliasParser;

impl<'a> Parse<'a> for TypeAliasParser {
    type Output = ItemKind;

    /// type <ident> "<" <generics> ">" = <type>
    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let generics = parser.parse_generics()?;
        parser.expect(TokenType::Eq)?;
        let ty = parser.parse_ty(false);
        parser.expect(TokenType::Semi)?;
        Ok(ItemKind::TypeAlias(generics, ty))
    }
}

enum FieldForm {
    Struct,
    Tuple,
}

pub struct FieldDeclParser {
    form: FieldForm,
}

impl<'a> Parse<'a> for FieldDeclParser {
    type Output = FieldDecl;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let vis = VisibilityParser.parse(parser)?;
        let ident = match self.form {
            FieldForm::Struct => {
                let ident = parser.expect_lident()?;
                parser.expect(TokenType::Colon)?;
                Some(ident)
            }
            FieldForm::Tuple => None,
        };
        let ty = parser.parse_ty(false);
        let span = vis.span.merge(ty.span);
        Ok(FieldDecl { id: parser.mk_id(), span, vis, ident, ty })
    }
}

pub struct VariantParser;

impl<'a> Parse<'a> for VariantParser {
    type Output = Variant;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let ident = parser.expect_uident()?;
        let kind = VariantKindParser.parse(parser)?;
        let span = ident.span.merge(parser.empty_span());
        Ok(Variant { id: parser.mk_id(), span, kind, ident })
    }
}

pub struct VariantKindParser;

impl<'a> Parse<'a> for VariantKindParser {
    type Output = VariantKind;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        if parser.accept(TokenType::OpenParen).is_some() {
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
            Ok(VariantKind::Unit)
        }
    }
}

pub struct StructDeclParser;

impl<'a> Parse<'a> for StructDeclParser {
    type Output = ItemKind;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let generics = GenericsParser.parse(parser)?;
        let kind = VariantKindParser.parse(parser)?;
        if let VariantKind::Tuple(_) | VariantKind::Unit = kind {
            parser.expect(TokenType::Semi)?;
        }
        Ok(ItemKind::Struct(generics, kind))
    }
}

pub struct EnumParser;

impl<'a> Parse<'a> for EnumParser {
    type Output = ItemKind;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let generics = GenericsParser.parse(parser)?;
        parser.expect(TokenType::OpenBrace)?;
        let variants =
            PunctuatedParser { inner: VariantParser, separator: TokenType::Comma }.parse(parser)?;
        parser.expect(TokenType::CloseBrace)?;
        Ok(ItemKind::Enum(generics, variants))
    }
}

pub struct FnParser;

impl<'a> Parse<'a> for FnParser {
    type Output = ItemKind;

    /// assumes that { <vis> fn <ident> } has already been parsed
    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let generics = GenericsParser.parse(parser)?;
        let sig = FnSigParser { require_type_annotations: true }.parse(parser)?;
        let block = if let Some(open_brace) = parser.accept(TokenType::OpenBrace) {
            Some(parser.parse_block(open_brace)?)
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
    macro parse($src:expr) {{
        let driver = ldriver::Driver::from_src($src);
        driver.parse().unwrap()
    }}

    #[test]
    fn parse_generics() {
        let _prog = parse!("fn test<T, U>() -> bool { false }");
    }

    #[test]
    fn parse_enum() {
        let _prog = parse!("enum B { T, F, }");
        let _prog = parse!("enum B { T(bool), F }");
        let _prog = parse!("enum B { T(bool), F { x: bool, y: &int } }");
    }

    #[test]
    fn parse_struct() {
        let _prog = parse!("struct S { x: int }");
        let _prog = parse!("struct S { x: int, y: bool }");
    }

    #[test]
    fn parse_tuple_struct() {
        let _prog = parse!("struct S(number);");
        let _prog = parse!("struct S(number, bool);");
    }
}

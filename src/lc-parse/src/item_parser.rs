use crate::*;
use lc_ast::*;
use lc_lex::TokenKind;
use lc_span::{with_source_map, ModuleKind};
use std::convert::TryFrom;

const ITEM_KEYWORDS: [TokenKind; 11] = [
    TokenKind::Fn,
    TokenKind::Macro,
    TokenKind::Struct,
    TokenKind::Enum,
    TokenKind::Const,
    TokenKind::Impl,
    TokenKind::Extern,
    TokenKind::Type,
    TokenKind::Use,
    TokenKind::Mod,
    TokenKind::Trait,
];

pub struct ItemParser;

impl<'a> Parse<'a> for ItemParser {
    type Output = P<Item>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let vis = VisibilityParser.parse(parser)?;

        // these items have a different syntax to the rest
        if let Some(_impl_kw) = parser.accept(TokenKind::Impl) {
            return ImplParser { vis }.parse(parser);
        } else if let Some(_extern_kw) = parser.accept(TokenKind::Extern) {
            return ExternParser { vis }.parse(parser);
        } else if let Some(_use_kw) = parser.accept(TokenKind::Use) {
            return UseParser { vis }.parse(parser);
        }

        if let Some(mod_kw) = parser.accept(TokenKind::Mod) {
            let name = parser.expect_lident()?;
            parser.expect(TokenKind::Semi)?;
            let span = mod_kw.span.merge(name.span);
            let module = SubModuleParser { span, name }.parse(parser)?;
            return Ok(parser.mk_item(vis.span.merge(name.span), vis, name, ItemKind::Mod(module)));
        }

        let kw = parser.expect_one_of(ITEM_KEYWORDS)?;
        let ident = parser.expect_ident()?;
        let (kind_span, kind) = parser.with_span(
            parse_fn(|parser| match kw.kind {
                TokenKind::Fn => FnParser.parse(parser),
                TokenKind::Macro => MacroItemParser.parse(parser),
                TokenKind::Struct => StructDeclParser.parse(parser),
                TokenKind::Enum => EnumParser.parse(parser),
                TokenKind::Type => TypeAliasParser.parse(parser),
                TokenKind::Trait => TraitParser.parse(parser),
                _ => unreachable!(),
            }),
            false,
        )?;

        parser.accept(TokenKind::Semi);

        Ok(parser.mk_item(vis.span.merge(kind_span), vis, ident, kind))
    }
}

pub struct ModuleParser;

impl<'a> Parse<'a> for ModuleParser {
    type Output = Module;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let mut items: Vec<Box<Item>> = vec![];
        while !parser.reached_eof() {
            items.push(ItemParser.parse(parser)?);
        }

        let span = if let Some(fst) = items.first() {
            let last = items.last().unwrap();
            fst.span.merge(last.span)
        } else {
            Span::default()
        };

        Ok(Module { span, items })
    }
}

pub struct SubModuleParser {
    /// the span of the declaration
    pub(crate) span: Span,
    /// the name of the module
    pub(crate) name: Ident,
}

impl<'a> Parse<'a> for SubModuleParser {
    type Output = Module;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        // disallow a `file module` having submodules
        if with_source_map(|map| {
            map.get_opt(parser.file).map(|src| src.file.kind) == Some(ModuleKind::File)
        }) {
            return Err(
                parser.build_err(self.span, ParseError::FileModuleWithSubmodules(self.name))
            );
        }

        let module_file = match with_source_map(|map| map.add_module(parser.file, *self.name)) {
            Some(file) => file,
            None =>
                return Err(parser.build_err(
                    self.span,
                    ParseError::UnresolvedModule(
                        with_source_map(|map| map.dir_of(parser.file).to_path_buf()),
                        self.name,
                    ),
                )),
        };

        with_source_map(|map| {
            debug!("found module `{}` at `{}`", self.name, map.path_of(module_file).display())
        });

        parser.with_file(module_file, |parser| ModuleParser.parse(parser))
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
        parser.expect(TokenKind::Semi)?;
        Ok(parser.mk_item(span, self.vis, Ident::empty(), kind))
    }
}

pub struct ExternParser {
    vis: Visibility,
}

impl<'a> Parse<'a> for ExternParser {
    type Output = P<Item>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let abi = parser.parse_abi()?;
        parser.expect(TokenKind::OpenBrace)?;
        let mut foreign_items = vec![];
        let close_brace = loop {
            if let Some(close_brace) = parser.accept(TokenKind::CloseBrace) {
                break close_brace;
            }
            let box Item { span, id, kind, vis, ident } = parser.parse_item()?;
            match ForeignItemKind::try_from(kind) {
                Ok(kind) => foreign_items.push(Box::new(Item { span, id, vis, ident, kind })),
                Err(kind) => parser.build_err(span, ParseError::InvalidForeignItem(kind)).emit(),
            };
        };

        let span = self.vis.span.merge(close_brace.span);

        let kind = ItemKind::Extern(abi, foreign_items);
        Ok(parser.mk_item(span, self.vis, Ident::empty(), kind))
    }
}

/// <vis> trait <ident><generics>? {
///     <trait-item>
/// }
pub struct TraitParser;

impl<'a> Parse<'a> for TraitParser {
    type Output = ItemKind;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        parser.expect(TokenKind::OpenBrace)?;
        let generics = parser.parse_generics()?;
        let items = ItemParser
            .many()
            .parse(parser)?
            .into_iter()
            .filter_map(|item| {
                let Item { span, id, vis, ident, .. } = *item;
                match AssocItemKind::try_from(item.kind) {
                    Ok(kind) => Some(Box::new(Item { span, id, vis, ident, kind })),
                    Err(kind) => {
                        parser.build_err(span, ParseError::InvalidTraitItem(kind)).emit();
                        None
                    }
                }
            })
            .collect();
        parser.expect(TokenKind::CloseBrace)?;
        Ok(ItemKind::Trait { generics, items })
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
        let self_ty = if parser.accept(TokenKind::For).is_some() {
            parser.parse_ty(false)
        } else {
            // reinterpret the trait path as the self type
            let ty_path = trait_path.take().unwrap();
            parser.mk_ty(ty_path.span, TyKind::Path(ty_path))
        };
        parser.expect(TokenKind::OpenBrace)?;
        let mut items = vec![];
        let close_brace = loop {
            if let Some(close_brace) = parser.accept(TokenKind::CloseBrace) {
                break close_brace;
            }
            let box Item { span, id, kind, vis, ident } = parser.parse_item()?;
            match AssocItemKind::try_from(kind) {
                Ok(kind) => items.push(Box::new(Item { span, id, vis, ident, kind })),
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
        parser.expect(TokenKind::Eq)?;
        let ty = parser.parse_ty(false);
        parser.expect(TokenKind::Semi)?;
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
                parser.expect(TokenKind::Colon)?;
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
        if parser.accept(TokenKind::OpenParen).is_some() {
            let form = FieldForm::Tuple;
            let fields = TupleParser { inner: FieldDeclParser { form } }.parse(parser)?;
            Ok(VariantKind::Tuple(fields))
        } else if parser.accept(TokenKind::OpenBrace).is_some() {
            let fields = PunctuatedParser {
                inner: FieldDeclParser { form: FieldForm::Struct },
                separator: TokenKind::Comma,
            }
            .parse(parser)?;
            parser.expect(TokenKind::CloseBrace)?;
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
            parser.expect(TokenKind::Semi)?;
        }
        Ok(ItemKind::Struct(generics, kind))
    }
}

pub struct EnumParser;

impl<'a> Parse<'a> for EnumParser {
    type Output = ItemKind;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let generics = GenericsParser.parse(parser)?;
        parser.expect(TokenKind::OpenBrace)?;
        let variants =
            PunctuatedParser { inner: VariantParser, separator: TokenKind::Comma }.parse(parser)?;
        parser.expect(TokenKind::CloseBrace)?;
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
        let block = if let Some(open_brace) = parser.accept(TokenKind::OpenBrace) {
            Some(parser.parse_block(open_brace)?)
        } else {
            parser.expect(TokenKind::Semi)?;
            None
        };
        let expr = block.map(|block| parser.mk_expr(block.span, ExprKind::Block(block)));
        Ok(ItemKind::Fn(sig, generics, expr))
    }
}

#[cfg(test)]
mod tests {
    macro parse($src:expr) {{
        let driver = lc_driver::Driver::from_src($src);
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

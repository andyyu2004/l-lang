use phf::phf_map;

use super::*;

impl<'a> Parser<'a> {
    fn parse_macro_matcher(&self) -> MacroMatcher {
        todo!()
    }
}

pub struct MacroParser;

impl<'a> Parse<'a> for MacroParser {
    type Output = ItemKind;

    /// assumes that { <vis> macro <ident> } has already been parsed
    ///
    /// <macro> ::= <vis> macro <ident> {
    ///    <macro-rule> +
    /// }
    /// <macro-rule> ::= <macro-matcher>* => { <macro-transcriber> }
    /// <macro-transcriber> ::= { <token-tree> }
    /// <token-tree> ::= <token>*
    /// <macro-repetition-operator> ::= * | + | ?
    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        parser.expect(TokenType::OpenBrace)?;
        let rules = todo!();
        let m = Macro { rules };
        Ok(ItemKind::Macro(m))
    }
}

struct MacroRuleParser;

impl<'a> Parse<'a> for MacroRuleParser {
    type Output = MacroRule;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        todo!()
    }
}

/// <macro-matcher> ::= ( <macro-match> )
/// <macro-match> ::=
///     | $<lident> : <macro-fragment-specifier>
///     | $( <macro-match>+ ) <macro-repetition-separator> <macro-repetition-operator>
///     | <macro-matcher>
///     | <token>
struct MacroMatcherParser;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FragmentSpecifier {
    Item,
    Block,
    Stmt,
    Pat,
    Expr,
    Ty,
    Ident,
    Path,
    Tt,
    Lit,
    Err,
}

pub enum MacroMatcher {
    Token(Token),
    Matcher(Box<MacroMatcher>),
    Fragment(Ident, FragmentSpecifier),
}

impl<'a> Parse<'a> for MacroMatcherParser {
    type Output = MacroMatcher;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        parser.expect(TokenType::OpenParen)?;
        let matcher = if parser.accept(TokenType::Dollar).is_some() {
            if let Some(ident) = parser.accept_lident() {
                parser.expect(TokenType::Colon)?;
                let specifier = MacroFragmentSpecifierParser.parse(parser)?;
                MacroMatcher::Fragment(ident, specifier)
            } else {
                MacroMatcher::Matcher(Box::new(Self.parse(parser)?))
            }
        } else {
            MacroMatcher::Token(parser.safe_peek()?)
        };
        parser.expect(TokenType::CloseParen)?;
        Ok(matcher)
    }
}

pub struct MacroFragmentSpecifierParser;

const MACRO_FRAGMENT_MAP: phf::Map<&'static str, FragmentSpecifier> = phf_map! {
    "item" => FragmentSpecifier::Item,
    "block" => FragmentSpecifier::Block,
    "stmt" => FragmentSpecifier::Stmt,
    "pat" => FragmentSpecifier::Pat,
    "expr" => FragmentSpecifier::Expr,
    "ty" => FragmentSpecifier::Ty,
    "ident" => FragmentSpecifier::Ident,
    "path" => FragmentSpecifier::Path,
    "tt" => FragmentSpecifier::Tt,
    "lit" => FragmentSpecifier::Lit,
};

impl<'a> Parse<'a> for MacroFragmentSpecifierParser {
    type Output = FragmentSpecifier;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let err_span = match parser.accept_lident() {
            Some(ident) => match MACRO_FRAGMENT_MAP.get(ident.as_str()) {
                Some(&spec) => return Ok(spec),
                None => ident.span,
            },
            None => parser.empty_span(),
        };
        parser.build_err(err_span, ParseError::MissingFragmentSpecifier).emit();
        Ok(FragmentSpecifier::Err)
    }
}

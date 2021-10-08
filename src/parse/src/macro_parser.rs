use super::*;
use phf::phf_map;

pub struct MacroParser;

impl<'a> Parse<'a> for MacroParser {
    type Output = ItemKind;

    /// assumes that `<vis> macro <ident>` has already been parsed
    ///
    /// <macro> ::= <vis> macro <ident> {
    ///    <macro-rule> +
    /// }
    /// <macro-rule> ::= <macro-matcher>* => { <macro-transcriber> }
    /// <macro-transcriber> ::= { <token-tree> }
    /// <token-tree> ::= <token>*
    /// <macro-repetition-operator> ::= * | + | ?
    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        parser.expect(TokenKind::OpenBrace)?;
        let rules = Punctuated1Parser { inner: MacroRuleParser, separator: TokenKind::Semi }
            .parse(parser)?;
        let m = Macro { rules };
        Ok(ItemKind::Macro(m))
    }
}

struct MacroRuleParser;

impl<'a> Parse<'a> for MacroRuleParser {
    type Output = MacroRule;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let matcher = MacroMatcherParser.parse(parser)?;
        parser.expect(TokenKind::RFArrow)?;
        let transcriber = parser.in_braces(parse_fn(|parser| parser.parse_tt()))?;
        Ok(MacroRule { matcher, transcriber })
    }
}

/// <macro-matcher> ::= ( <macro-match> )
/// <macro-match> ::=
///     | $<lident> : <macro-fragment-specifier>
///     | $( <macro-match>+ ) <macro-repetition-separator> <macro-repetition-operator>
///     | <macro-matcher>
///     | <token>
struct MacroMatcherParser;

impl<'a> Parse<'a> for MacroMatcherParser {
    type Output = MacroMatcher;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        parser.expect(TokenKind::OpenParen)?;
        let matcher = if parser.accept(TokenKind::Dollar).is_some() {
            if let Some(ident) = parser.accept_lident() {
                parser.expect(TokenKind::Colon)?;
                let specifier = MacroFragmentSpecifierParser.parse(parser)?;
                MacroMatcher::Fragment(ident, specifier)
            } else {
                MacroMatcher::Matcher(Box::new(Self.parse(parser)?))
            }
        } else {
            MacroMatcher::Token(parser.safe_peek()?)
        };
        parser.expect(TokenKind::CloseParen)?;
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

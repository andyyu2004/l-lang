use super::*;
use phf::phf_map;

pub struct MacroItemParser;

impl<'a> Parse<'a> for MacroItemParser {
    type Output = ItemKind;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        parser.parse_macro().map(ItemKind::Macro)
    }
}

pub struct MacroParser;

impl<'a> Parse<'a> for MacroParser {
    type Output = Macro;

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
        let rules = parser.in_braces(MacroRuleParser.punctuated1(TokenKind::Semi))?;
        Ok(Macro { rules })
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
///     | <token> (except delimiters)

struct MacroMatcherParser;

impl<'a> Parse<'a> for MacroMatcherParser {
    type Output = MacroMatcher;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        // TODO need custom logic rather than using `many` as we need to stop when we hit a delimiter
        let matches = parser.in_parens(MacroMatchParser.many())?;
        Ok(MacroMatcher { matches })
    }
}

struct MacroMatchParser;

impl<'a> Parse<'a> for MacroMatchParser {
    type Output = MacroMatch;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let matcher = if parser.accept(TokenKind::Dollar).is_some() {
            if let Some(ident) = parser.accept_lident() {
                parser.expect(TokenKind::Colon)?;
                let specifier = MacroFragmentSpecifierParser.parse(parser)?;
                MacroMatch::Fragment(ident, specifier)
            } else {
                let matches = parser.in_parens(self.many1())?;
                let (sep, repetitor) = match MacroRepetitorParser.parse(parser) {
                    Ok(rep) => (None, rep),
                    Err(_) => {
                        let sep = parser.safe_next().ok();
                        let rep = MacroRepetitorParser.parse(parser)?;
                        (sep, rep)
                    }
                };
                MacroMatch::Repetition(matches, sep, repetitor)
            }
        } else {
            // The NOT $ cases
            let token = parser.safe_peek()?;
            // TODO The caller must ensure this?
            assert!(!token.kind.is_delimiter());
            MacroMatch::Token(parser.next())
        };
        Ok(matcher)
    }
}

pub struct MacroRepetitorParser;

impl<'a> Parse<'a> for MacroRepetitorParser {
    type Output = MacroRepetitor;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        parser.expect_one_of([TokenKind::Plus, TokenKind::Star, TokenKind::Question]).map(|token| {
            match token.kind {
                TokenKind::Plus => MacroRepetitor::Plus,
                TokenKind::Star => MacroRepetitor::Star,
                TokenKind::Question => MacroRepetitor::Opt,
                _ => unreachable!(),
            }
        })
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

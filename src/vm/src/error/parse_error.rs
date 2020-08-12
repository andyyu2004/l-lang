use crate::lexer::{Tok, TokenType};
use crate::span::Span;
use thiserror::Error;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    pub span: Span,
    pub kind: ParseErrorKind,
}

impl ParseError {
    fn new(span: Span, kind: ParseErrorKind) -> Self {
        Self { span, kind }
    }

    pub fn  unexpected_eof(span: Span) -> Self {
        Self { span, kind: ParseErrorKind::Eof }
    }

    pub fn  expected_semi(span: Span) -> Self {
        Self { span, kind: ParseErrorKind::MissingSemi }
    }

    pub fn  expected(ttype: TokenType, found: Tok) -> Self {
        Self::new(found.span, ParseErrorKind::Expected(ttype, found.ttype))
    }

    pub fn  require_type_annotations(span: Span) -> Self {
        Self::new(span, ParseErrorKind::RequireTypeAnnotations)
    }

    pub fn  expected_one_of(ttypes: Vec<TokenType>, found: Tok) -> Self {
        Self::new(found.span, ParseErrorKind::ExpectedOneOf(ttypes, found.ttype))
    }

    pub fn  unimpl() -> Self {
        Self::new(Span { lo: 0, hi: 0 }, ParseErrorKind::Unimpl)
    }
}

#[derive(Debug, Error)]
pub enum ParseErrorKind {
    #[error("expected `{0:?}` found `{1:?}`")]
    Expected(TokenType, TokenType),
    #[error("expected one of `{0:?}` found `{1:?}`")]
    ExpectedOneOf(Vec<TokenType>, TokenType),
    #[error("unexpected <eof>")]
    Eof,
    #[error("function signature requires explicit type annotations")]
    RequireTypeAnnotations,
    #[error("expected semicolon after expression statement")]
    MissingSemi,
    #[error("unimplemented")]
    Unimpl,
}

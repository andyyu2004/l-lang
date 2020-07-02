use crate::lexer::{Span, Tok, TokenKind};
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

    pub fn expected(kind: TokenKind, found: Tok) -> Self {
        Self::new(found.span, ParseErrorKind::Expected(kind, found.kind))
    }
}

#[derive(Debug, Error)]
pub enum ParseErrorKind {
    #[error("expected `{0:?}` found `{1:?}`")]
    Expected(TokenKind, TokenKind),
    #[error("found <eof>")]
    Eof,
}

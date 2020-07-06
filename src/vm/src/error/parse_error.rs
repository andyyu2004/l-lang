use crate::lexer::{Tok, TokenType};
use crate::span::Span;
use thiserror::Error;

crate type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    pub span: Span,
    pub kind: ParseErrorKind,
}

impl ParseError {
    fn new(span: Span, kind: ParseErrorKind) -> Self {
        Self { span, kind }
    }

    crate fn expected(ttype: TokenType, found: Tok) -> Self {
        Self::new(found.span, ParseErrorKind::Expected(ttype, found.ttype))
    }
}

#[derive(Debug, Error)]
pub enum ParseErrorKind {
    #[error("expected `{0:?}` found `{1:?}`")]
    Expected(TokenType, TokenType),
    #[error("found <eof>")]
    Eof,
}

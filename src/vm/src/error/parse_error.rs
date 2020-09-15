use super::DiagnosticBuilder;
use crate::ast::{Expr, P};
use crate::lexer::{Tok, TokenType};
use crate::span::Span;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

pub type ParseResult<'a, T> = Result<T, DiagnosticBuilder<'a>>;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("expected `{0:?}` found `{:?}`", .1.ttype)]
    Expected(TokenType, Tok),
    #[error("expected one of `{0:?}` found `{:?}`", .1.ttype)]
    ExpectedOneOf(Vec<TokenType>, Tok),
    #[error("unexpected <eof>")]
    Eof,
    #[error("function signature requires explicit type annotations")]
    RequireTypeAnnotations,
    #[error("expected semicolon after expression statement")]
    MissingSemi,
    #[error("unsafe operation requries unsafe context")]
    RequireUnsafeCtx,
    #[error("unimplemented in parser")]
    Unimpl,
}

use super::DiagnosticBuilder;
use crate::ast::{Expr, ItemKind, P};
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
    #[error("invalid impl item kind {}", .0.descr())]
    InvalidImplItem(ItemKind),
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
    #[error("generic arguments not allowed in module paths")]
    GenericArgsInModulePath,
    #[error("generic arguments in expression path require `::` prefix")]
    AmbiguousGenericArgsInExprPath,
    #[error("elided type annotation not allowed here")]
    ElidedTypeNotAllowedInThisContext,
}
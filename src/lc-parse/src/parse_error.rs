use lc_ast::{Ident, ItemKind};
use lc_error::DiagnosticBuilder;
use lc_lex::{DelimiterKind, Token, TokenKind};
use lc_span::Symbol;
use std::path::PathBuf;
use thiserror::Error;

pub type ParseResult<'a, T> = Result<T, DiagnosticBuilder<'a>>;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("expected `{0:?}` found `{}`", .1.kind)]
    Expected(TokenKind, Token),
    #[error("invalid abi `{0}`\nvalid abi's include \"l\", \"l-instrinsic")]
    InvalidAbi(String),
    #[error("expected one of `{0:?}` found `{}`", .1.kind)]
    ExpectedOneOf(Vec<TokenKind>, Token),
    #[error("invalid impl item kind: {}", .0.descr())]
    InvalidImplItem(ItemKind),
    #[error("invalid trait item kind: {}", .0.descr())]
    InvalidTraitItem(ItemKind),
    #[error("invalid foreign item kind: {}", .0.descr())]
    InvalidForeignItem(ItemKind),
    #[error("unresolved module `{0}`\ncreate file at either `{0}/{1}.l` or `{0}/{1}/{1}.l`")]
    UnresolvedModule(PathBuf, Ident),
    #[error("modules declared as a file cannot have submodules")]
    FileModuleWithSubmodules(Ident),
    #[error("expected uppercase identifier, found `{0}`")]
    ExpectUppercaseIdentifier(Symbol),
    #[error("expected lowercase identifier, found `{0}`")]
    ExpectLowercaseIdentifier(Symbol),
    #[error("expected literal, found `{0}`")]
    ExpectedLiteral(TokenKind),
    #[error("unexpected <eof>")]
    Eof,
    #[error("function signature requires explicit type annotations")]
    RequireTypeAnnotations,
    #[error("expected semicolon after expression statement")]
    MissingSemi,
    #[error("unimplemented in parser")]
    Unimpl,
    #[error("redundant visibility modifier")]
    RedundantVisibilityModifier,
    #[error("generic arguments not allowed in module paths")]
    GenericArgsInModulePath,
    #[error("elided type annotation not allowed here")]
    ElidedTypeNotAllowedInThisContext,
    #[error("unterminated string literal")]
    UnterminatedStringLiteral,
    #[error("missing fragment specifier")]
    MissingFragmentSpecifier,
    #[error("expected token `{0}` to close group, found `{1}`")]
    MismatchedTokenTreeDelimiter(TokenKind, TokenKind),
    #[error("unmatched opening delimiter `{0}`")]
    UnmatchedOpenTokenTreeDelimiter(DelimiterKind),
}

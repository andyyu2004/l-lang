use ast::{Ident, Path, PathSegment};
use ir::DefKind;
use span::Span;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("unresolved value path segment `{0}` in path `{1}`")]
    UnresolvedPath(PathSegment, Path),
    #[error("unresolved type `{0}`")]
    UnresolvedType(Path),
    #[error("let binding to named closure")]
    BindingToNamedClosure,
    #[error("{0} with name `{1}` already defined")]
    DuplicateDefinition(DefKind, Ident),
    #[error("module with name `{0}` already defined")]
    DuplicateModuleDefinition(Ident),
    #[error("identifier `{0}` bound more than once in the same pattern")]
    DuplicatePatternIdentifier(Ident),
}

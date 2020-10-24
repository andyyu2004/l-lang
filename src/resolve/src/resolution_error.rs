use ast::{Ident, Path, PathSegment};
use error::DiagnosticBuilder;
use thiserror::Error;

pub type ResResult<'a, T> = Result<T, DiagnosticBuilder<'a>>;

#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("unresolved value path segment `{0}` in path `{1}`")]
    UnresolvedPath(PathSegment, Path),
    #[error("unresolved type `{0}`")]
    UnresolvedType(Path),
    #[error("let binding to named closure")]
    BindingToNamedClosure,
    #[error("item with name `{0}` already defined")]
    DuplicateDefinition(Ident),
    #[error("module with name `{0}` already defined")]
    DuplicateModuleDefinition(Ident),
    #[error("identifier `{0}` bound more than once in the same pattern")]
    DuplicatePatternIdentifier(Ident),
}

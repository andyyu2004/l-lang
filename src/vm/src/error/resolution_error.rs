use crate::ast::Path;
use crate::span::Span;
use thiserror::Error;

pub type ResolutionResult<T> = Result<T, ResolutionError>;

#[derive(Debug)]
pub struct ResolutionError {
    span: Span,
    kind: ResolutionErrorKind,
}

impl ResolutionError {
    pub fn unbound_variable(path: Path) -> Self {
        Self { span: path.span, kind: ResolutionErrorKind::UnresolvedPath(path) }
    }

    pub fn unknown_type(path: Path) -> Self {
        Self { span: path.span, kind: ResolutionErrorKind::UnresolvedType(path) }
    }
}

#[derive(Debug, Error)]
pub enum ResolutionErrorKind {
    #[error("Unresolved path `{0}`")]
    UnresolvedPath(Path),
    #[error("Unresolved type `{0}`")]
    UnresolvedType(Path),
}

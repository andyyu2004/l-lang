use crate::ast::Path;
use crate::span::Span;
use thiserror::Error;

crate type ResolutionResult<T> = Result<T, ResolutionError>;

#[derive(Debug)]
crate struct ResolutionError {
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
crate enum ResolutionErrorKind {
    #[error("Unresolved path `{0}`")]
    UnresolvedPath(Path),
    #[error("Unresolved type `{0}`")]
    UnresolvedType(Path),
}

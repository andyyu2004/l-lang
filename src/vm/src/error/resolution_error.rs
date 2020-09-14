use crate::ast::Path;
use crate::span::Span;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("Unresolved path `{0}`")]
    UnresolvedPath(Path),
    #[error("Unresolved type `{0}`")]
    UnresolvedType(Path),
}

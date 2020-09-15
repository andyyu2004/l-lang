use crate::ast::{Path, PathSegment};
use crate::span::Span;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("Unresolved path `{0}` in `{1}`")]
    UnresolvedPath(PathSegment, Path),
    #[error("Unresolved type `{0}`")]
    UnresolvedType(Path),
}

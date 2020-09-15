use crate::ast::{Path, PathSegment};
use crate::span::Span;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("unresolved path `{0}` in `{1}`")]
    UnresolvedPath(PathSegment, Path),
    #[error("unresolved type `{0}`")]
    UnresolvedType(Path),
    #[error("let binding to named closure")]
    BindingToNamedClosure,
}

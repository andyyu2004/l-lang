use lc_span::Span;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum MirError {
    #[error("use of uninitialized variable `{}`", .0.to_string())]
    UninitializedVariable(Span),
    #[error("assignment to immutable variable `{}`", .0.to_string())]
    AssignmentToImmutableVar(Span),
}

use crate::ast::Ident;
use crate::ir;
use crate::span::Span;
use crate::ty::Ty;
use thiserror::Error;

pub type TypeResult<'tcx, T> = Result<T, TypeError<'tcx>>;

#[derive(Debug, Error)]
pub enum TypeError<'tcx> {
    #[error("failed to unify type `{0}` with `{1}`")]
    UnificationFailure(Ty<'tcx>, Ty<'tcx>),
    #[error("expected type `{0}`, found `{1}`")]
    Mismatch(Ty<'tcx>, Ty<'tcx>),
    #[error("expected `{0}-tuple`, found `{1}-tuple`")]
    TupleSizeMismatch(usize, usize),
    #[error("unknown field `{1}` on `{0}`")]
    UnknownField(Ty<'tcx>, Ident),
    #[error("attempted to index `.{0}` into a {1}-tuple")]
    TupleOutOfBounds(usize, usize),
    #[error("{0}")]
    Msg(String),
    #[error("`main` has type `{0}` but should be of type `fn() -> int`")]
    IncorrectMainType(Ty<'tcx>),
    #[error("field `{0}` already declared in `{1}`")]
    FieldAlreadyDeclared(Ident, Ident),
    #[error("type annotations required")]
    InferenceFailure,
}

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
    #[error("{0}")]
    Msg(String),
    #[error("type annotations required")]
    InferenceFailure,
}

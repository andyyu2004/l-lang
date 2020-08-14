use crate::ir;
use crate::ty::Ty;
use thiserror::Error;

pub type TypeResult<'tcx, T> = Result<T, TypeError<'tcx>>;

#[derive(Debug, Error)]
pub enum TypeError<'tcx> {
    #[error("Failed to unify type `{0}` with `{1}`")]
    UnificationFailure(Ty<'tcx>, Ty<'tcx>),
    #[error("Expected type `{0}`, found `{1}`")]
    Mismatch(Ty<'tcx>, Ty<'tcx>),
    #[error("Expected `{0}-tuple`, found `{1}-tuple`")]
    TupleSizeMismatch(usize, usize),
    #[error("Require type annotations")]
    InferenceFailure,
    #[error("{0}")]
    Msg(String),
}

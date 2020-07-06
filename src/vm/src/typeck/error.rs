use crate::ty::Ty;
use thiserror::Error;

crate type TypeResult<'tcx, T> = Result<T, TypeError<'tcx>>;
crate type InferResult<'tcx, T> = Result<T, InferError<'tcx>>;

#[derive(Debug, Error)]
crate enum TypeError<'tcx> {
    #[error("{0}")]
    InferError(InferError<'tcx>),
}

#[derive(Debug, Error, Copy, Clone)]
crate enum InferError<'tcx> {
    #[error("Failed to unify type `{0}` with `{1}`")]
    UnificationFailure(Ty<'tcx>, Ty<'tcx>),
    #[error("Require type annotations")]
    InferenceFailure,
}

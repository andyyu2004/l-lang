use crate::ty::Ty;
use thiserror::Error;

crate type TypeResult<'tcx, T> = Result<T, TypeError<'tcx>>;

#[derive(Debug, Error)]
crate enum TypeError<'tcx> {
    #[error("Failed to unify type `{0}` with `{1}`")]
    UnificationFailure(Ty<'tcx>, Ty<'tcx>),
    #[error("Require type annotations")]
    InferenceFailure,
}

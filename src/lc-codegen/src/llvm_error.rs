use lc_core::ty::Ty;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LLVMError<'tcx> {
    #[error("missing `main` function")]
    MissingMain,
    #[error("main function must have type `fn() -> int`, found {0}")]
    InvalidMainType(Ty<'tcx>),
    #[error("function `main` defined twice")]
    DuplicateMain,
}

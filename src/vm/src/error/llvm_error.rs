use thiserror::Error;

#[derive(Error, Debug)]
pub enum LLVMError {
    #[error("missing `main` function")]
    MissingMain,
}

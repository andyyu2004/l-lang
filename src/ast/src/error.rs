use thiserror::Error;

#[derive(Debug, Error)]
pub enum AstError {
    #[error("functions must have a body")]
    FunctionWithoutBody,
}

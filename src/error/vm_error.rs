use thiserror::Error;

pub type VMResult<T> = Result<T, VMError>;

#[derive(Debug, Error)]
pub enum VMError {
    #[error("invalid opcode `{0}`")]
    InvalidOpcode(u8),
    #[error("invalid type`{0}`")]
    InvalidType(u8),
}

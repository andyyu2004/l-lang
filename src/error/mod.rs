mod parse_error;
mod vm_error;

pub use parse_error::{ParseError, ParseResult};
pub use vm_error::{VMError, VMResult};

pub type LResult<T> = Result<T, LError>;

#[derive(Debug)]
pub enum LError {
    VMError(VMError),
    ParseError(ParseError),
}

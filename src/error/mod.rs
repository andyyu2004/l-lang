mod parse_error;
mod vm_error;

use crate::impl_from_inner;
pub use parse_error::{ParseError, ParseResult};
pub use vm_error::{VMError, VMResult};

pub type LResult<T> = Result<T, LError>;

impl_from_inner!(VMError, LError, VMError);
impl_from_inner!(ParseError, LError, ParseError);

#[derive(Debug)]
pub enum LError {
    VMError(VMError),
    ParseError(ParseError),
}

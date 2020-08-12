mod diagnostic;
mod diagnostic_builder;
mod emitter;
mod parse_error;
mod resolution_error;
mod type_error;
mod vm_error;

use crate::impl_from_inner;
pub use diagnostic::{Diagnostic, Diagnostics};
pub use diagnostic_builder::DiagnosticBuilder;
pub use emitter::{Emitter, TextEmitter};
pub use parse_error::{ParseError, ParseResult};
pub use resolution_error::{ResolutionError, ResolutionResult};
pub use type_error::{TypeError, TypeResult};
pub use vm_error::{VMError, VMResult};

pub type LResult<T> = Result<T, LError>;

impl_from_inner!(VMError, LError, VMError);
impl_from_inner!(ParseError, LError, ParseError);

#[derive(Debug)]
pub enum LError {
    VMError(VMError),
    ParseError(ParseError),
    Error(String),
    ErrorReported,
}

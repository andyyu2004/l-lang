mod diagnostic;
mod diagnostic_builder;
mod emitter;
mod llvm_error;
mod parse_error;
mod resolution_error;
mod type_error;
mod vm_error;

use crate::impl_from_inner;
pub use diagnostic::{Diagnostic, Diagnostics};
pub use diagnostic_builder::DiagnosticBuilder;
pub use emitter::{Emitter, TextEmitter};
pub use llvm_error::LLVMError;
pub use parse_error::{ParseError, ParseResult};
pub use resolution_error::{ResolutionError, ResolutionResult};
pub use type_error::{TypeError, TypeResult};
pub use vm_error::{VMError, VMResult};

pub type LResult<T> = Result<T, LError>;

impl_from_inner!(VMError, LError, VMError);

#[derive(Debug)]
pub enum LError {
    VMError(VMError),
    ErrorReported,
}

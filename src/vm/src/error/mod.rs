mod diagnostic;
mod diagnostic_builder;
mod emitter;
mod parse_error;
mod resolution_error;
mod type_error;
mod vm_error;

use crate::impl_from_inner;
crate use diagnostic::{Diagnostic, Diagnostics};
crate use diagnostic_builder::DiagnosticBuilder;
crate use emitter::{Emitter, TextEmitter};
crate use parse_error::{ParseError, ParseResult};
crate use resolution_error::{ResolutionError, ResolutionResult};
crate use type_error::{TypeError, TypeResult};
crate use vm_error::{VMError, VMResult};

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

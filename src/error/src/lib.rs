#![feature(box_syntax)]
#![feature(crate_visibility_modifier)]
#![feature(min_specialization)]

mod diagnostics;

pub use codespan_reporting::diagnostic::Severity;
pub use diagnostics::*;

pub type LResult<T> = Result<T, ErrorReported>;

#[derive(Debug, Clone, Copy)]
pub struct ErrorReported;

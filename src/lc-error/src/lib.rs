#![feature(min_specialization)]

mod diagnostics;

pub use codespan_reporting::diagnostic::Severity;
pub use diagnostics::*;

pub type LResult<T> = Result<T, ErrorReported>;

#[derive(Debug, Clone, Copy)]
pub struct ErrorReported;

pub trait LError: std::error::Error {
    ///(very) approximate description of the error
    ///details can go into the `Display` impl
    fn title(&self) -> &str;
}

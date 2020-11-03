#![feature(box_syntax)]
#![feature(crate_visibility_modifier)]

mod diagnostics;

pub use diagnostics::*;

pub type LResult<T> = Result<T, LError>;

#[derive(Debug)]
pub enum LError {
    ErrorReported,
}

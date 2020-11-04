#![feature(box_syntax)]
#![feature(crate_visibility_modifier)]
#![feature(min_specialization)]

mod diagnostics;

pub use diagnostics::*;

pub type LResult<T> = Result<T, LError>;

#[derive(Debug)]
pub enum LError {
    ErrorReported,
}

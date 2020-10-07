#![feature(box_syntax)]

#[macro_use]
extern crate colour;

mod diagnostics;

pub use diagnostics::*;

pub type LResult<T> = Result<T, LError>;

#[derive(Debug)]
pub enum LError {
    ErrorReported,
}

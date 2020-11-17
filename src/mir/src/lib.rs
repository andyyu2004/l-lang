//! mir analyses and optimizations

#![feature(decl_macro)]
#![feature(crate_visibility_modifier)]
#![feature(box_syntax, box_patterns)]

#[macro_use]
extern crate log;

mod dataflow;
mod opt;
mod typecheck;

pub use dataflow::analyze;
pub use opt::{early_opt, late_opt};
pub use typecheck::typecheck;

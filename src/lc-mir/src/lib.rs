//! mir analyses and optimizations

#![feature(decl_macro)]
#![feature(box_patterns)]

extern crate log;

#[cfg(test)]
mod tests;

mod const_eval;
mod dataflow;
mod opt;
mod typecheck;

pub use dataflow::analyze;
use lc_core::queries::Queries;
pub use opt::{early_opt, late_opt};
pub use typecheck::typecheck;

pub fn provide(queries: &mut Queries) {
    const_eval::provide(queries);
}

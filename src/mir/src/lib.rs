//! mir analyses and optimizations

#![feature(decl_macro)]
#![feature(crate_visibility_modifier)]
#![feature(box_syntax, box_patterns)]

#[macro_use]
extern crate log;

#[cfg(test)]
mod tests;

mod const_eval;
mod dataflow;
mod opt;
mod typecheck;

pub use dataflow::analyze;
use lcore::queries::Queries;
pub use opt::{early_opt, late_opt};
pub use typecheck::typecheck;

pub fn provide(queries: &mut Queries) {
    const_eval::provide(queries);
}

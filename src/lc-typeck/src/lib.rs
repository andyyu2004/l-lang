#![feature(decl_macro)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate lc_util;

extern crate lc_ir as ir;

#[cfg(test)]
mod tests;

mod autoderef;
mod check;
mod coherence;
pub mod collect;
mod tyconv;
mod type_of;
mod upvars;
mod writeback;

pub use check::{FnCtx, InheritedCtx};
pub use tyconv::TyConv;

use autoderef::Autoderef;
use lc_core::queries::Queries;

pub fn provide(queries: &mut Queries) {
    collect::provide(queries);
    check::provide(queries);
    type_of::provide(queries);
}

#![feature(decl_macro)]
#![feature(crate_visibility_modifier)]
#![feature(or_patterns)]

#[macro_use]
extern crate log;

#[cfg(test)]
mod tests;

mod autoderef;
mod check;
mod collect;
mod expr;
mod pat;
mod stmt;
mod tir;
mod tyconv;
mod type_of;
mod upvars;
mod writeback;

pub use crate::tir::build_tir;
use autoderef::Autoderef;
pub use check::{typeck_fn, FnCtx};
pub use collect::collect_item_types;
pub use tyconv::TyConv;
pub use type_of::Typeof;

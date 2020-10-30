#![feature(decl_macro)]
#![feature(array_value_iter)]
#![feature(crate_visibility_modifier)]
#![feature(or_patterns)]

#[macro_use]
extern crate log;

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

use autoderef::Autoderef;
pub use check::{FnCtx, InheritedCtx};
pub use collect::TcxCollectExt;
pub use tyconv::TyConv;
pub use type_of::{TcxTypeofExt, Typeof};

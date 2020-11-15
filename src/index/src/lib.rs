#![feature(allow_internal_unstable)]
#![feature(bool_to_option)]
#![feature(const_fn)]
#![feature(const_panic)]
#![feature(extend_one)]
#![feature(unboxed_closures)]
#![feature(test)]
#![feature(fn_traits)]

#[macro_use]
extern crate serde;

pub mod indexvec;

pub use indexvec::{Idx, IndexVec};

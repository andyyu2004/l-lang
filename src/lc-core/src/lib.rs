#![feature(hash_raw_entry)]
#![feature(const_mut_refs)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(crate_visibility_modifier)]
#![feature(extern_types)]
#![feature(const_panic)]
#![feature(const_raw_ptr_deref)]
#![feature(decl_macro)]
#![feature(type_name_of_val)]

#[macro_use]
extern crate serde_derive;

extern crate lc_ir as ir;

mod arena;
mod defmap;
mod interners;
pub mod mir;
pub mod queries;
pub mod ty;

pub use crate::arena::{Arena, ArenaAllocatable};
pub use ty::{GlobalCtx, TyCtx};

use interners::CtxInterners;
use queries::Queries;

pub fn provide(queries: &mut Queries) {
    ty::provide(queries);
}

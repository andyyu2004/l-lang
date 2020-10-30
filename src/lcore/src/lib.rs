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
extern crate log;

mod arena;
mod interners;
mod ir_map;
pub mod mir;
pub mod queries;
pub mod ty;

pub use crate::arena::{Arena, ArenaAllocatable};
use interners::CtxInterners;
pub use ty::{GlobalCtx, TyCtx};

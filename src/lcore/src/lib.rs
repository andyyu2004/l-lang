#![feature(hash_raw_entry)]
#![feature(extern_types)]
#![feature(const_panic)]
#![feature(const_raw_ptr_deref)]
#![feature(decl_macro)]
#![feature(type_name_of_val)]

mod arenas;
mod interners;
mod ir_map;
pub mod mir;
pub mod ty;
mod tyctx;

pub use arenas::CoreArenas;
use interners::CtxInterners;
pub use tyctx::{GlobalCtx, TyCtx};

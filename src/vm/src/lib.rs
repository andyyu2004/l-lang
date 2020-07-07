#![feature(type_name_of_val)]
#![feature(box_syntax, box_patterns)]
#![feature(iterator_fold_self)]
#![feature(raw)]
#![feature(hash_set_entry)]
#![feature(extern_types)]
#![feature(box_into_raw_non_null)]
#![feature(concat_idents)]
#![feature(btree_drain_filter)]
#![feature(crate_visibility_modifier)]
#![feature(core_intrinsics)]
#![feature(dropck_eyepatch)]
#![feature(raw_vec_internals)]
#![allow(dead_code)]

#[macro_use]
extern crate derive_deref;

#[macro_use]
extern crate vm_derive;

mod arena;
mod ast;
mod compiler;
mod driver;
mod error;
mod exec;
mod gc;
mod ir;
mod lexer;
mod parser;
mod shared;
mod span;
mod tir;
mod ty;
mod typeck;
mod util;

use driver::Driver;
use error::LResult;

pub fn exec(src: &str) -> LResult<()> {
    let driver = Driver::new(src);
    let expr = driver.gen_tir_expr()?;
    Ok(())
}

// in tir every expression has a type (i.e. a ty field))
// mod tir;
// in ir, only some places will be typed
// mod ir;

#[cfg(test)]
mod test {
    use crate::exec::*;

    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Val>(), 16);
    }
}

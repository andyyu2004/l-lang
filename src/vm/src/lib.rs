#![feature(type_name_of_val)]
#![feature(box_syntax, box_patterns)]
#![feature(debug_non_exhaustive)]
#![feature(associated_type_defaults)]
#![feature(iterator_fold_self)]
#![feature(raw)]
#![feature(const_panic)]
#![feature(hash_set_entry)]
#![feature(extern_types)]
#![feature(box_into_raw_non_null)]
#![feature(concat_idents)]
#![feature(decl_macro)]
#![feature(btree_drain_filter)]
#![feature(crate_visibility_modifier)]
#![feature(core_intrinsics)]
#![feature(dropck_eyepatch)]
#![feature(raw_vec_internals)]
#![feature(const_fn)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_macros)]

#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate indexed_vec;
#[macro_use]
extern crate vm_derive;
#[macro_use]
extern crate log;

mod arena;
mod ast;
mod compiler;
mod core;
mod driver;
mod error;
mod exec;
mod gc;
mod ir;
mod lexer;
mod parser;
mod resolve;
mod span;
mod tir;
mod ty;
mod typeck;
mod util;

use driver::Driver;
use error::LResult;
use log::LevelFilter;

pub fn exec(src: &str) -> LResult<()> {
    simple_logging::log_to_file("log.txt", LevelFilter::Info).unwrap();
    let driver = Driver::new(src);
    let res = driver.exec()?;
    println!("{}", res);
    Ok(())
}

// just stupidly wraps the expr string in a function to form a program
fn wrap_in_main(src: &str) -> String {
    format!("fn main() {{ {} }}", src)
}

pub fn exec_expr(src: &str) -> LResult<()> {
    let src = wrap_in_main(src);
    exec(&src)
}

#[cfg(test)]
mod test {
    use crate::exec::*;

    /// ensure this doesn't unintentionally get larger
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Val>(), 16);
    }
}

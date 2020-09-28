#![feature(type_name_of_val)]
#![feature(or_patterns)]
#![feature(box_syntax, box_patterns)]
#![feature(is_sorted)]
#![feature(bindings_after_at)]
#![feature(crate_visibility_modifier)]
#![recursion_limit = "256"]
#![feature(refcell_take)]
#![feature(split_inclusive)]
#![feature(hash_raw_entry)]
#![feature(const_raw_ptr_deref)]
#![feature(type_ascription)]
#![feature(debug_non_exhaustive)]
#![feature(associated_type_defaults)]
#![feature(iterator_fold_self)]
#![feature(raw)]
#![feature(const_panic)]
#![feature(hash_set_entry)]
#![feature(extern_types)]
#![feature(concat_idents)]
#![feature(decl_macro)]
#![feature(btree_drain_filter)]
#![feature(core_intrinsics)]
#![feature(dropck_eyepatch)]
#![feature(raw_vec_internals)]
#![feature(array_value_iter)]
#![feature(const_fn)]
#![feature(never_type)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_macros)]

#[macro_use]
extern crate smallvec;
#[macro_use]
extern crate colour;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate indexed_vec;
#[macro_use]
extern crate vm_derive;
#[macro_use]
extern crate log;

pub mod arena;
pub mod ast;
pub mod compiler;
pub mod core;
pub mod dataflow;
pub mod driver;
pub mod error;
pub mod exec;
pub mod gc;
pub mod ir;
pub mod jit;
pub mod lexer;
pub mod llvm;
pub mod mir;
pub mod parser;
pub mod resolve;
pub mod span;
pub mod tir;
pub mod ty;
pub mod typeck;
pub mod util;

use driver::Driver;
use error::LResult;
use log::LevelFilter;

fn mk_driver(src: &str) -> Driver {
    simple_logging::log_to_file("log.txt", LevelFilter::Info).unwrap();
    Driver::new(src)
}

pub fn parse(src: &str) -> LResult<Box<ast::Prog>> {
    let driver = mk_driver(src);
    driver.parse()
}

// pub fn exec(src: &str) -> LResult<exec::Val> {
//     let driver = mk_driver(src);
//     let res = driver.exec()?;
//     Ok(res)
// }

pub fn jit(src: &str) -> LResult<i32> {
    let driver = mk_driver(src);
    driver.llvm_jit()
}

/// runs all analyses but produces no output
pub fn check(src: &str) -> LResult<()> {
    let driver = mk_driver(src);
    driver.check()
}

pub fn dump_tir(src: &str) -> LResult<()> {
    let driver = mk_driver(src);
    println!("{}", driver.gen_tir()?);
    Ok(())
}

pub fn llvm_exec(src: &str) -> LResult<i32> {
    let driver = mk_driver(src);
    driver.llvm_exec()
}

pub macro tir($src:expr) {{
    let driver = Driver::new($src);
    driver.gen_tir().unwrap()
}}

// just stupidly wraps the expr string in an int function to form a program
fn wrap_in_main(src: &str) -> String {
    format!("fn main() -> int {{ {} }}", src)
}

pub fn llvm_exec_expr(src: &str) -> LResult<i32> {
    let src = wrap_in_main(src);
    llvm_exec(&src)
}

// pub fn exec_expr(src: &str) -> LResult<exec::Val> {
//     let src = wrap_in_main(src);
//     exec(&src)
// }

#[cfg(test)]
mod test {
    use crate::exec::*;

    /// ensure this doesn't unintentionally get larger
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Val>(), 16);
    }
}

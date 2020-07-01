#![feature(type_name_of_val)]
#![feature(box_syntax)]
#![feature(raw)]
#![feature(box_into_raw_non_null)]
#![feature(concat_idents)]
#![feature(btree_drain_filter)]
#![feature(crate_visibility_modifier)]

mod ast;
pub mod compiler;
mod driver;
mod error;
pub mod exec;
mod gc;
mod lexer;
mod parser;
mod util;

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

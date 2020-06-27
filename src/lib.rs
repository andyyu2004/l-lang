#![feature(type_name_of_val)]
#![feature(box_syntax)]
#![feature(raw)]
#![feature(box_into_raw_non_null)]
#![feature(concat_idents)]
#![feature(btree_drain_filter)]

pub mod compiler;
mod error;
pub mod exec;
mod gc;
mod util;

#[cfg(test)]
mod test {
    use crate::exec::*;

    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Val>(), 16);
    }
}

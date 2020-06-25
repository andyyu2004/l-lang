#![feature(type_name_of_val)]
#![feature(box_syntax)]
#![feature(raw)]
#![feature(box_into_raw_non_null)]

mod compiler;
mod error;
mod exec;
mod gc;
mod util;

pub use exec::*;

#[cfg(test)]
mod test {
    use crate::exec::*;
    use crate::gc::*;

    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Val>(), 16);
        assert_eq!(std::mem::size_of::<Gc<Obj>>(), 8);
    }
}

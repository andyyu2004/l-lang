use super::Array;
use crate::gc::{GCStateMap, Trace};
use crate::{impl_as_mut, Function};

/// it seems to be easier to put the variants directly into value instead (wrt the Gc pointers)
/// also makes casting less nested
#[derive(Debug)]
pub enum Obj {
    Array(Array),
    Fn(Function),
}

impl_as_mut!(Obj, as_array, Array, Array);
impl_as_mut!(Obj, as_fn, Fn, Function);

impl Trace for Obj {
    fn mark(&self, map: &mut GCStateMap) {
        match self {
            Self::Array(array) => array.mark(map),
            Self::Fn(f) => f.mark(map),
        }
    }
}

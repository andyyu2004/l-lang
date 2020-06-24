use super::Array;
use gc::{GCStateMap, Trace};

#[derive(Debug)]
pub enum Obj {
    Array(Array),
}

impl Obj {
    pub fn as_array(&mut self) -> &mut Array {
        match self {
            Self::Array(array) => array,
            // _ => panic!("expected array found `{:?}`", self),
        }
    }
}

impl Trace for Obj {
    fn mark(&self, map: &mut GCStateMap) {
        match self {
            Self::Array(array) => array.mark(map),
        }
    }
}

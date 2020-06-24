use super::Obj;
use crate::from_inner;
use crate::gc::{GCStateMap, Gc, Trace};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Val {
    Obj(Gc<Obj>),
    Prim(u64),
    Unit,
}

impl Default for Val {
    fn default() -> Self {
        Self::Prim(0)
    }
}

impl Val {
    pub fn as_prim(&self) -> u64 {
        match *self {
            Self::Prim(u) => u,
            _ => panic!("expected obj found `{:?}`", self),
        }
    }

    pub fn as_obj(&self) -> Gc<Obj> {
        match *self {
            Self::Obj(obj) => obj,
            _ => panic!("expected primitive value found `{:?}`", self),
        }
    }
}

impl Trace for Val {
    fn mark(&self, map: &mut GCStateMap) {
        match self {
            Self::Obj(ptr) => ptr.mark(map),
            Self::Prim(_) | Self::Unit => {}
        }
    }
}

from_inner!(Gc<Obj>, Val, Obj);
from_inner!(u64, Val, Prim);

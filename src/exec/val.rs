use super::Obj;
use crate::from_inner;
use gc::Gc;

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

from_inner!(Gc<Obj>, Val, Obj);
from_inner!(u64, Val, Prim);

use crate::gc::{GCStateMap, Gc, Trace};
use crate::{impl_from_inner, impl_into, Array, Closure, Function};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Val {
    Array(Gc<Array>),
    Fn(Gc<Function>),
    Clsr(Gc<Closure>),
    Str(Gc<String>),
    Prm(u64),
    Unit,
}

impl Default for Val {
    fn default() -> Self {
        Self::Prm(0)
    }
}

impl_into!(Val, Prm, u64);
impl_into!(Val, Array, Gc<Array>);
impl_into!(Val, Fn, Gc<Function>);

impl_from_inner!(Gc<Function>, Val, Fn);
impl_from_inner!(Gc<Closure>, Val, Clsr);
impl_from_inner!(Gc<Array>, Val, Array);
impl_from_inner!(u64, Val, Prm);

impl Val {
    pub fn as_prm(&self) -> u64 {
        (*self).into()
    }

    pub fn as_array(&self) -> Gc<Array> {
        (*self).into()
    }

    pub fn as_fn(&self) -> Gc<Function> {
        (*self).into()
    }
}

impl Trace for Val {
    fn mark(&self, map: &mut GCStateMap) {
        match self {
            Self::Fn(f) => f.mark(map),
            Self::Clsr(f) => f.mark(map),
            Self::Array(xs) => xs.mark(map),
            Self::Str(s) => s.mark(map),
            Self::Prm(_) | Self::Unit => {}
        }
    }
}

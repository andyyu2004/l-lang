use crate::exec::{Array, Closure, Function};
use crate::gc::{GCStateMap, Gc, Trace};
use crate::{impl_from_inner, impl_into};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Val {
    Array(Gc<Array>),
    Fn(Gc<Function>),
    Clsr(Gc<Closure>),
    Str(Gc<String>),
    UInt(u64),
    Int(i64),
    Double(f64),
    Unit,
}

impl Default for Val {
    fn default() -> Self {
        Self::Unit
    }
}

impl_into!(Val, Int, i64);
impl_into!(Val, UInt, u64);
impl_into!(Val, Double, f64);
impl_into!(Val, Array, Gc<Array>);
impl_into!(Val, Fn, Gc<Function>);

impl_from_inner!(Gc<Function>, Val, Fn);
impl_from_inner!(Gc<Closure>, Val, Clsr);
impl_from_inner!(Gc<Array>, Val, Array);
impl_from_inner!(u64, Val, UInt);
impl_from_inner!(i64, Val, Int);
impl_from_inner!(f64, Val, Double);

impl Val {
    pub fn as_u64(&self) -> u64 {
        (*self).into()
    }

    pub fn as_f64(&self) -> f64 {
        (*self).into()
    }

    pub fn as_i64(&self) -> i64 {
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
            Self::Double(_) | Self::Int(_) | Self::UInt(_) | Self::Unit => {}
        }
    }
}

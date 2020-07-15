use super::{Data, Instance};
use crate::exec::obj;
use crate::exec::*;
use crate::gc::{GCStateMap, Gc, Trace};
use crate::{impl_from_inner, impl_into};
use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Val {
    Array(Gc<Array>),
    Fn(Gc<Function>),
    Data(Gc<Data>),
    Instance(Gc<Instance>),
    Clsr(Gc<Closure>),
    Str(Gc<String>),
    Tuple(Gc<obj::Tuple>),
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

impl Display for Val {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Val::Array(x) => write!(f, "{:?}", x),
            Val::Fn(x) => write!(f, "{:?}", x),
            Val::Data(x) => write!(f, "{:?}", x),
            Val::Instance(x) => write!(f, "{:?}", x),
            Val::Clsr(x) => write!(f, "{:?}", x),
            Val::Str(x) => write!(f, "{:?}", x),
            Val::Tuple(x) => write!(f, "{}", x),
            Val::UInt(u) => write!(f, "{}", u),
            Val::Int(i) => write!(f, "{}", i),
            Val::Double(d) => write!(f, "{}", d),
            Val::Unit => write!(f, "()"),
        }
    }
}

impl_into!(Val, Int, i64);
impl_into!(Val, UInt, u64);
impl_into!(Val, Double, f64);
impl_into!(Val, Array, Gc<Array>);
impl_into!(Val, Fn, Gc<Function>);

impl_from_inner!(Gc<Function>, Val, Fn);
impl_from_inner!(Gc<Data>, Val, Data);
impl_from_inner!(Gc<Instance>, Val, Instance);
impl_from_inner!(Gc<Closure>, Val, Clsr);
impl_from_inner!(Gc<Array>, Val, Array);
impl_from_inner!(Gc<Tuple>, Val, Tuple);
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
            Self::Data(d) => Gc::mark(d, map),
            Self::Instance(d) => Gc::mark(d, map),
            Self::Fn(f) => Gc::mark(f, map),
            Self::Clsr(f) => Gc::mark(f, map),
            Self::Array(xs) => Gc::mark(xs, map),
            Self::Str(s) => Gc::mark(s, map),
            Self::Tuple(t) => Gc::mark(t, map),
            Self::Double(_) | Self::Int(_) | Self::UInt(_) | Self::Unit => {}
        }
    }
}

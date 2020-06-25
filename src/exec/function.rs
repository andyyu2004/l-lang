use super::Code;
use crate::gc::{GCStateMap, Trace};
use std::ptr::NonNull;

#[derive(Debug)]
pub struct Function {
    pub(crate) code: Code,
}

impl Function {
    pub fn new(code: Code) -> Self {
        Self { code }
    }
}

impl Trace for Function {
}

#[derive(Debug)]
pub struct Closure {
    /// ptr to inner function;
    /// it is not the closure's responsibility to free up the function as it does not own it
    /// therefore, it is not a gc pointer to it
    f: NonNull<Function>,
}

impl Closure {
    pub fn new(f: NonNull<Function>) -> Self {
        Self { f }
    }
}

impl Trace for Closure {
    fn mark(&self, map: &mut GCStateMap) {
    }
}

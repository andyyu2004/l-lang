use super::{Function, Upval};
use crate::gc::{GCStateMap, Gc, Trace};

#[derive(Debug)]
pub struct Closure {
    pub f: Gc<Function>,
    pub upvals: Vec<Gc<Upval>>,
}

impl Closure {
    pub fn new(f: Gc<Function>) -> Self {
        Self {
            f,
            upvals: Vec::with_capacity(f.upvalc as usize),
        }
    }
}

impl Trace for Closure {
    fn mark(&self, map: &mut GCStateMap) {
        Gc::mark(&self.f, map);
        self.upvals.iter().for_each(|u| Gc::mark(u, map));
    }
}

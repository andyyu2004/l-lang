use super::{Function, Upvar};
use crate::gc::{GCStateMap, Gc, Trace};

#[derive(Debug)]
pub struct Closure {
    pub f: Gc<Function>,
    pub upvars: Vec<Gc<Upvar>>,
}

impl Closure {
    pub fn new(f: Gc<Function>) -> Self {
        Self {
            f,
            upvars: Vec::with_capacity(f.upvalc as usize),
        }
    }
}

impl Trace for Closure {
    fn mark(&self, map: &mut GCStateMap) {
        Gc::mark(&self.f, map);
        self.upvars.iter().for_each(|u| Gc::mark(u, map));
    }
}

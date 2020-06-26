use super::{Function, Upval};
use crate::gc::{GCStateMap, Gc, Trace};

#[derive(Debug)]
pub struct Closure {
    pub f: Gc<Function>,
    pub upvals: Vec<Upval>,
}

impl Closure {
    pub fn new(f: Gc<Function>) -> Self {
        Self { f, upvals: vec![] }
    }
}

impl Trace for Closure {
    fn mark(&self, map: &mut GCStateMap) {
        self.f.mark(map);
        self.upvals.iter().for_each(|u| u.mark(map));
    }
}

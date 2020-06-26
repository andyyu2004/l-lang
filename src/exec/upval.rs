use crate::gc::{GCStateMap, Trace};

#[derive(Debug)]
pub struct Upval {}

impl Trace for Upval {
    fn mark(&self, map: &mut GCStateMap) {
    }
}

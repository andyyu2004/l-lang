use crate::gc::{GCStateMap, Trace};

/// runtime representation of a data object
#[derive(Clone, Debug, PartialEq)]
pub struct Data {}

impl Trace for Data {
    fn mark(&self, _map: &mut GCStateMap) {
    }
}

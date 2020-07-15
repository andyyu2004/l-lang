use super::{Data, Val};
use crate::gc::{GCStateMap, Gc, Trace};

/// instantiation of a some class
#[derive(Clone, Debug, PartialEq)]
pub struct Instance {
    data: Gc<Data>,
    fields: Vec<Gc<Val>>,
}

impl Instance {
    pub fn new(data: Gc<Data>) -> Self {
        Self { data, fields: Default::default() }
    }
}

impl Trace for Instance {
    fn mark(&self, map: &mut GCStateMap) {
        Gc::mark(&self.data, map);
        self.fields.iter().for_each(|field| Gc::mark(field, map));
    }
}

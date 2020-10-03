//! runtime representations of heap allocated vm objects

use super::Val;
use crate::gc::{GCStateMap, Gc, Trace};
use crate::util;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Tuple {
    elements: Vec<Val>,
}

impl Trace for Tuple {
    fn mark(&self, map: &mut GCStateMap) {
        self.elements.mark(map)
    }
}

impl Tuple {
    pub fn new(elements: Vec<Val>) -> Self {
        Self { elements }
    }
}

impl Display for Tuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({})", util::join(&self.elements, ","))
    }
}

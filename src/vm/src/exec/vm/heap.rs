use crate::gc::{GarbageCollector, Gc, Trace};
use std::cell::Cell;

#[derive(Default, Debug)]
pub struct Heap<G> {
    pub(super) gc: G,
    pub(super) disabled: Cell<bool>,
}

impl<G> Heap<G>
where
    G: GarbageCollector,
{
    pub fn new(gc: G) -> Self {
        Self { gc, disabled: Cell::new(false) }
    }

    pub fn disable_gc(&self) {
        self.disabled.set(true)
    }

    pub fn enable_gc(&self) {
        self.disabled.set(false)
    }

    pub fn alloc_and_gc<T>(&mut self, t: T, root: impl Trace) -> Gc<T>
    where
        T: Trace + 'static,
    {
        if !self.disabled.get() {
            self.gc.mark_sweep(root);
        }
        self.gc.alloc(t)
    }
}

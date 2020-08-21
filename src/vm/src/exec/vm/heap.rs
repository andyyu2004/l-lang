use crate::exec::{Closure, Function};
use crate::gc::{GarbageCollector, Gc, Trace};
use std::cell::Cell;

#[derive(Default, Debug)]
pub struct Heap<G> {
    pub(super) gc: G,
    pub(super) disabled: Cell<bool>,
}

impl<'tcx, G> Heap<G>
where
    G: GarbageCollector<'tcx>,
{
    pub fn new(gc: G) -> Self {
        Self { gc, disabled: Cell::new(false) }
    }

    fn without_gc<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.disable_gc();
        let ret = f(self);
        self.enable_gc();
        ret
    }

    pub fn mk_clsr(&mut self, f: Function) -> Gc<Closure> {
        self.without_gc(|heap| {
            let f = heap.gc.alloc(f);
            heap.gc.alloc(Closure::new(f))
        })
    }

    pub fn disable_gc(&self) {
        self.disabled.set(true)
    }

    pub fn enable_gc(&self) {
        self.disabled.set(false)
    }

    pub fn alloc_and_gc<T>(&mut self, t: T, root: impl Trace) -> Gc<T>
    where
        T: Trace + 'tcx,
    {
        if !self.disabled.get() {
            self.gc.mark_sweep(root);
        }
        self.gc.alloc(t)
    }
}

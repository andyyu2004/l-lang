use super::{Gc, Trace};

pub trait GarbageCollector {
    fn alloc<T>(&mut self, t: T) -> Gc<T>
    where
        T: Trace + 'static;

    fn mark_sweep(&mut self, root: impl Trace);
}

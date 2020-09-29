use super::{Gc, Trace};
use std::alloc::Layout;

pub trait GarbageCollector<'tcx> {
    fn alloc<T>(&mut self, t: T) -> Gc<T>
    where
        T: Trace + 'tcx;

    fn mark_sweep(&mut self, root: impl Trace);
}

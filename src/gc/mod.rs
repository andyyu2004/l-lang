mod garbage_collector;
mod gc;
mod gc_ptr;
mod trace;

pub use garbage_collector::GarbageCollector;
pub use gc::GC;
pub use gc_ptr::Gc;
pub use trace::{GCStateMap, Trace};

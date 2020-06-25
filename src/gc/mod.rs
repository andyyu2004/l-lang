mod gc;
mod gc2;
mod gc_ptr;
mod trace;

pub use gc::GC;
pub use gc2::GC as GC2;
pub use gc_ptr::Gc;
pub use trace::{GCStateMap, Trace};

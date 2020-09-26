//! native functions

use super::Runtime;
use crate::gc::GarbageCollector;

#[no_mangle]
fn alloc_new<'tcx, G>(runtime: &Runtime<G>) -> *mut u64
where
    G: GarbageCollector<'tcx>,
{
    todo!();
    // runtime.gc.alloc(todo!()).ptr.as_ptr()
}

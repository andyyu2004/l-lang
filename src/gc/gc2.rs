use super::{GCStateMap, Gc, Trace};
use alloc::Layout;
use rustc_hash::FxHashSet;
use std::alloc;
use std::ptr::NonNull;

/// the garbage collector
/// not to be confused with Gc
#[derive(Debug, Default)]
pub struct GC {
    allocated_bytes: usize,
    allocated: FxHashSet<NonNull<dyn Trace>>,
    #[cfg(debug_assertions)]
    pub dbg_allocations: Vec<Option<NonNull<dyn Trace>>>,
}

impl GC {
    fn alloc_t<T: ?Sized>(t: &T) -> *mut u8 {
        unsafe { alloc::alloc(Layout::for_value(t)) }
    }

    fn dealloc_t<T: ?Sized>(ptr: NonNull<T>) {
        let r = unsafe { ptr.as_ref() };
        unsafe { alloc::dealloc(ptr.as_ptr().cast(), Layout::for_value(r)) }
    }

    pub fn alloc<T>(&mut self, t: T) -> Gc<T>
    where
        T: Trace + 'static,
    {
        self.allocated_bytes += std::mem::size_of_val(&t);
        let ptr = Self::alloc_t(&t).cast::<T>();
        unsafe { std::ptr::write(ptr, t) };
        let ptr = NonNull::new(ptr).unwrap();
        #[cfg(debug_assertions)]
        self.dbg_allocations.push(Some(ptr));
        self.allocated.insert(ptr);
        Gc::new(ptr)
    }

    pub fn mark_sweep(&mut self, root: impl Trace) {
        let mut reachable = GCStateMap::default();
        root.mark(&mut reachable);

        // free all references that are allocated but not marked and unmark the marked ones
        let mut to_release = FxHashSet::default();
        for &ptr in &self.allocated {
            // pointer used for comparison
            let cmp_ptr = ptr.cast();

            // if p was reached during mark phase
            if reachable.contains(&cmp_ptr) {
                continue;
            }

            // what about the Gc<T> itself?
            // otherwise free the pointer
            to_release.insert(ptr);
            println!("freeing ptr: {:?}", ptr);
            self.allocated_bytes -= std::mem::size_of_val(unsafe { &*ptr.as_ptr() });
            Self::dealloc_t(ptr)
        }

        // set the deallocated pointers to None
        #[cfg(debug_assertions)]
        for mptr in &mut self.dbg_allocations {
            if let Some(ptr) = mptr {
                if !reachable.contains(&ptr.cast()) {
                    *mptr = None;
                }
            }
        }

        // retain ptr iff it is not to be released
        self.allocated.retain(|ptr| !to_release.contains(ptr));
    }
}

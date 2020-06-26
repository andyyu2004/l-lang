use super::{GCStateMap, GarbageCollector, Gc, Trace};
use alloc::Layout;
use rustc_hash::FxHashSet;
use std::alloc;
use std::ptr::NonNull;

/// the garbage collector
/// not to be confused with the Gc ptr
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
}

impl GarbageCollector for GC {
    fn alloc<T>(&mut self, t: T) -> Gc<T>
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

    fn mark_sweep(&mut self, root: impl Trace) {
        let mut reachable = GCStateMap::default();
        root.mark(&mut reachable);

        // free all references that are allocated but not marked and unmark the marked ones
        let mut to_release = FxHashSet::default();
        for &ptr in &self.allocated {
            // pointer used for comparison
            let cmp_ptr: NonNull<()> = ptr.cast();

            // if p was reached during mark phase
            if reachable.contains(&cmp_ptr) {
                continue;
            }

            // otherwise free the pointer
            to_release.insert(ptr);
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

#[cfg(test)]
mod test {
    use super::*;

    struct VM {
        pub stack: Vec<u64>,
    }

    impl Default for VM {
        fn default() -> Self {
            Self { stack: vec![] }
        }
    }

    impl Trace for VM {
        fn mark(&self, map: &mut GCStateMap) {
            for &ptr in &self.stack {
                let raw = ptr as *mut Gc<Obj>;
                unsafe { raw.as_ref() }.unwrap().mark(map);
            }
        }
    }

    #[derive(Debug)]
    struct Obj {
        x: usize,
    }

    impl Trace for Obj {
    }

    #[test]
    fn simple_alloc() {
        let mut gc = GC::default();
        let x = 5;
        let ptr = gc.alloc(Obj { x });
        assert_eq!(ptr.x, 5);
    }

    /// expect gc to not free value as there is a reference from the stack
    #[test]
    fn run_simple_mark_sweep() {
        let mut gc = GC::default();
        let mut vm = VM::default();
        let gc_ptr: Gc<Obj> = gc.alloc(Obj { x: 5 });
        vm.stack.push(&gc_ptr as *const Gc<Obj> as u64);
        gc.mark_sweep(vm);
    }
}

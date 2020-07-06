use super::{GCStateMap, Trace};
use std::ops::{Deref, DerefMut};
use std::{hash::Hash, ptr::NonNull};

/// garbage collected ptr type
/// not to be confused with GC
#[derive(Debug)]
pub struct Gc<T>
where
    T: ?Sized,
{
    pub(crate) ptr: NonNull<T>,
}

impl<T> Copy for Gc<T> {
}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr.clone(),
        }
    }
}

impl<T> Hash for Gc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.hash(state)
    }
}

impl<T> Eq for Gc<T> {
}

impl<T> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> Gc<T> {
    pub fn new(ptr: NonNull<T>) -> Self {
        Self { ptr }
    }
}

impl<T> Trace for Gc<T>
where
    T: Trace + 'static,
{
    fn mark(&self, map: &mut GCStateMap) {
        if map.contains(&self.ptr.cast()) {
            return;
        }
        self.ptr.mark(map);
        map.mark_gc_ptr(self);
    }
}

impl<T> Deref for Gc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr.as_ptr() }
    }
}

impl<T> DerefMut for Gc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

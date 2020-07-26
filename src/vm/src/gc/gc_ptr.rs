use super::{GCStateMap, Trace};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// garbage collected ptr type
/// not to be confused with `GC`
pub struct Gc<T>
where
    T: ?Sized,
{
    pub(crate) ptr: NonNull<T>,
}

impl<T: Debug> Debug for Gc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let inner = unsafe { &*self.ptr.as_ptr() };
        write!(f, "{:?}", inner)
    }
}

impl<T: Display> Display for Gc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let inner = unsafe { &*self.ptr.as_ptr() };
        write!(f, "{}", inner)
    }
}

impl<T> Copy for Gc<T> {
}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T> Hash for Gc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
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

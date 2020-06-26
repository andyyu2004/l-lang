use super::Val;
use crate::gc::{GCStateMap, Gc, Trace};
use std::{
    ops::{Deref, DerefMut}, ptr::NonNull
};

#[derive(Debug)]
pub enum Upval {
    /// points to the value on the stack
    Open(NonNull<Val>),
    /// moved onto the heap
    Closed(Gc<Val>),
}

impl Deref for Upval {
    type Target = Val;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Open(ptr) => unsafe { ptr.as_ref() },
            Self::Closed(ptr) => ptr,
        }
    }
}

impl DerefMut for Upval {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Open(ptr) => unsafe { ptr.as_mut() },
            Self::Closed(ptr) => ptr,
        }
    }
}

impl Trace for Upval {
    fn mark(&self, map: &mut GCStateMap) {
        if let Self::Closed(ptr) = self {
            ptr.mark(map)
        }
    }
}

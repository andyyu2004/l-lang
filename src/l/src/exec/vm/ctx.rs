use crate::exec::{Closure, Frame, Upvar, Val};
use crate::gc::{GCStateMap, Gc, Trace};
use std::{collections::BTreeMap, mem, ptr::NonNull};

const FRAMES_MAX: usize = 64;
pub const STACK_MAX: usize = FRAMES_MAX * (std::u8::MAX as usize + 1);

/// contains the fields that need to be gced
pub struct VMCtx {
    /// base pointer; points to where in the stack the current frame starts (i.e. the index of the
    /// currently executing function ptr)
    pub(crate) bp: usize,
    /// frame pointer to the index of the current frame in frames;
    pub(crate) fp: usize,
    /// value stack
    pub(crate) stack: [Val; STACK_MAX],
    /// call stack
    pub(crate) frames: [Frame; FRAMES_MAX],
    pub(crate) constants: Vec<Val>,
    /// map from a stack address to the upvalue that captures the value at that address
    /// there will only be one upvalue as it can be reused
    pub(crate) open_upvars: BTreeMap<NonNull<Val>, Gc<Upvar>>,
}

impl Trace for VMCtx {
    fn mark(&self, map: &mut GCStateMap) {
        let sp = self.frames[self.fp - 1].sp;
        self.stack[..self.bp + sp].iter().for_each(|val| val.mark(map));
        self.frames[..self.fp].iter().for_each(|frame| frame.mark(map));
        self.constants.iter().for_each(|val| val.mark(map));
        self.open_upvars.values().for_each(|ptr| Gc::mark(ptr, map));
    }
}
impl VMCtx {
    pub(crate) fn new(f: Gc<Closure>, constants: Vec<Val>) -> Self {
        // safety: we will never access the unintialized memory before explicitly setting the frame
        const N: usize = FRAMES_MAX * mem::size_of::<Frame>() / mem::size_of::<u32>();
        let mut frames: [Frame; FRAMES_MAX] = unsafe { mem::transmute([0u32; N]) };
        frames[0] = Frame::new(f, 0);
        let stack = [Val::default(); STACK_MAX];

        Self { stack, frames, constants, open_upvars: Default::default(), fp: 1, bp: 0 }
    }
}

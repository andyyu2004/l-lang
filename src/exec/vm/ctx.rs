use crate::exec::{Closure, Frame, Val};
use crate::gc::{GCStateMap, Gc, Trace};
use std::mem;

const FRAMES_MAX: usize = 4;
const STACK_MAX: usize = 8;
// const STACK_MAX: usize = FRAMES_MAX * (std::u8::MAX as usize + 1);

/// contains the fields that need to be gced
pub struct Ctx {
    /// base pointer; points to where in the stack the current frame starts (i.e. the index of the
    /// currently executing function ptr)
    pub(crate) bp: usize,
    /// frame pointer to the index of the current frame in frames;
    pub(crate) fp: usize,
    pub(crate) stack: [Val; STACK_MAX],
    pub(crate) frames: [Frame; FRAMES_MAX],
    pub(crate) constants: Vec<Val>,
}

impl Trace for Ctx {
    fn mark(&self, map: &mut GCStateMap) {
        let sp = self.frames[self.fp - 1].sp;
        self.stack[..self.bp + sp]
            .iter()
            .for_each(|val| val.mark(map));
        self.frames[..self.fp]
            .iter()
            .for_each(|frame| frame.mark(map));
        self.constants.iter().for_each(|val| val.mark(map));
    }
}
impl Ctx {
    pub(crate) fn new(f: Gc<Closure>, constants: Vec<Val>) -> Self {
        // safety: we will never access the unintialized memory before explicitly setting the frame
        const N: usize = FRAMES_MAX * mem::size_of::<Frame>() / mem::size_of::<u32>();
        let mut frames: [Frame; FRAMES_MAX] = unsafe { mem::transmute([0u32; N]) };
        frames[0] = Frame::new(f, 0);
        let stack = [Val::default(); STACK_MAX];

        Self {
            stack,
            frames,
            constants,
            fp: 1,
            bp: 0,
        }
    }
}

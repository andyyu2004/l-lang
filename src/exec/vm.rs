use super::*;
use crate::error::*;
use std::convert::TryFrom;
use std::mem;

const FRAMES_MAX: usize = 1;
const STACK_MAX: usize = FRAMES_MAX * (std::u8::MAX as usize + 1);

pub struct VM {
    /// frame pointer to the index of the current frame
    /// note this is different to the stack pointer (in frame) which points to the top of the stack
    fp: usize,
    frames: [Frame; FRAMES_MAX],
    stack: [u64; STACK_MAX],
}

impl VM {
    pub fn new(main: Function) -> Self {
        // safety: we will never access the unintialized memory before explicitly setting the frame
        const N: usize = FRAMES_MAX * mem::size_of::<Frame>() / mem::size_of::<u32>();
        let mut frames: [Frame; FRAMES_MAX] = unsafe { mem::transmute([0u32; N]) };
        frames[0] = Frame::new(main);
        Self {
            fp: 0,
            frames,
            stack: [0u64; STACK_MAX],
        }
    }

    pub fn run(&mut self) -> VMResult<u64> {
        let frame = &mut self.frames[self.fp];
        macro_rules! push {
            ($value:expr) => {{
                self.stack[self.fp + frame.sp] = $value;
                frame.sp += 1;
            }};
        }

        macro_rules! pop {
            () => {{
                frame.sp -= 1;
                self.stack[self.fp + frame.sp]
            }};
        }

        macro_rules! arith {
            ($op: tt, $ty:ty) => {{
                // don't inline as the evaluation order needs to be this way (pop second operand first)
                let r = pop!() as $ty;
                let l = pop!() as $ty;
                push!((l $op r) as u64)
            }}
        }

        macro_rules! farith {
            ($op: tt) => {{
                let r = f64::from_bits(pop!());
                let l = f64::from_bits(pop!());
                push!((l $op r) as u64)
            }}
        }

        Ok(loop {
            match frame.read_opcode()? {
                Op::nop => {}
                Op::iconst | Op::uconst | Op::dconst => push!(frame.read_u64()),
                Op::iadd => arith!(+, i64),
                Op::uadd => arith!(+, u64),
                Op::dadd => farith!(+),
                Op::isub => arith!(-, i64),
                Op::usub => arith!(-, u64),
                Op::dsub => farith!(-),
                Op::imul => arith!(*, i64),
                Op::umul => arith!(*, u64),
                Op::dmul => farith!(*),
                Op::idiv => arith!(/, i64),
                Op::udiv => arith!(/, u64),
                Op::ddiv => farith!(/),
                // how to differentiate return types :o
                Op::iret | Op::uret | Op::dret => break pop!(),
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// tests the vm applies arithmetic left to right
    #[test]
    fn vm_arith_order() -> VMResult<()> {
        let code = CodeBuilder::default()
            .add_op(Op::iconst)
            .add_const(7i64)
            .add_op(Op::iconst)
            .add_const(5i64)
            .add_op(Op::isub)
            .add_op(Op::iret)
            .build();
        let main = Function::new(code);
        let mut vm = VM::new(main);
        let ret = vm.run()?;
        assert_eq!(ret, 2);
        Ok(())
    }
}

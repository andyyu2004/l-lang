use super::*;
use crate::compiler::{Constant, ConstantPool, Executable};
use crate::error::*;
use crate::gc::{GCStateMap, Gc, Trace, GC};
use std::mem;

const FRAMES_MAX: usize = 4;
const STACK_MAX: usize = FRAMES_MAX * (std::u8::MAX as usize + 1);

#[derive(Default, Debug)]
pub struct Heap {
    gc: GC,
}

impl Heap {
    pub fn alloc_and_gc<T>(&mut self, t: T, root: impl Trace) -> Gc<T>
    where
        T: Trace + 'static,
    {
        self.gc.mark_sweep(root);
        self.gc.alloc(t)
    }
}

/// contains the fields that need to be gced
pub struct Ctx {
    /// frame pointer to the index of the current frame in frames;
    fp: usize,
    stack: [Val; STACK_MAX],
    frames: [Frame; FRAMES_MAX],
    constants: Vec<Val>,
}

impl Trace for Ctx {
    fn mark(&self, map: &mut GCStateMap) {
        self.stack.iter().for_each(|val| val.mark(map));
        self.frames.iter().for_each(|frame| frame.mark(map));
        self.constants.iter().for_each(|val| val.mark(map));
    }
}
impl Ctx {
    fn new(f: Gc<Function>, constants: Vec<Val>) -> Self {
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
        }
    }
}

/// the virtual machine
pub struct VM {
    /// base pointer; points to where in the stack the current frame starts (i.e. the index of the
    /// currently executing function ptr)
    bp: usize,
    ctx: Ctx,
    heap: Heap,
}

impl VM {
    pub fn new(executable: Executable) -> Self {
        let mut heap = Heap::default();

        let Executable { constants, start } = executable;
        let f = heap.gc.alloc(start);
        let constants = constants
            .into_iter()
            .map(|c| match c {
                Constant::Function(f) => Val::Fn(heap.gc.alloc(f)),
                Constant::String(s) => Val::Str(heap.gc.alloc(s)),
            })
            .collect();

        Self {
            bp: 0,
            heap,
            ctx: Ctx::new(f, constants),
        }
    }

    pub fn run(&mut self) -> VMResult<Val> {
        let mut frame = &mut self.ctx.frames[self.ctx.fp - 1] as *mut Frame;
        macro_rules! frame {
            () => {
                unsafe { &*frame }
            };
        }

        macro_rules! frame_mut {
            () => {
                unsafe { &mut *frame }
            };
        }

        macro_rules! push {
            ($value:expr) => {{
                self.ctx.stack[self.bp + frame!().sp] = $value.into();
                frame_mut!().sp += 1;
            }};
        }

        macro_rules! pop {
            () => {{
                frame_mut!().sp -= 1;
                self.ctx.stack[self.bp + frame!().sp]
            }};
        }

        macro_rules! arith {
            ($op: tt, $ty:ty) => {{
                // don't inline as the evaluation order needs to be this way (pop second operand first)
                let r = pop!().as_prm() as $ty;
                let l = pop!().as_prm() as $ty;
                let res = (l $op r) as u64;
                push!(res)
            }}
        }

        macro_rules! farith {
            ($op: tt) => {{
                let r = f64::from_bits(pop!().as_prm());
                let l = f64::from_bits(pop!().as_prm());
                let res = (l $op r) as u64;
                push!(res)
            }}
        }

        macro_rules! astore {
            ($ty:ty) => {{
                let value = pop!().as_prm() as $ty;
                let index = pop!().as_prm() as isize;
                let array_ref = pop!();
                array_ref.as_array().set::<$ty>(index, value);
            }};
        }

        Ok(loop {
            // println!("{:?}", &self.ctx.stack[..frame!().sp]);
            match frame_mut!().read_opcode()? {
                Op::nop => {}
                Op::iconst | Op::uconst | Op::dconst => push!(frame_mut!().read_u64()),
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
                Op::pop => frame_mut!().sp -= 1,
                // how to differentiate return types?
                Op::ret | Op::uret | Op::dret | Op::iret => {
                    let ret = pop!();
                    self.ctx.fp -= 1;
                    if self.ctx.fp == 0 {
                        break ret;
                    }
                    self.bp = frame!().ret_addr;
                    frame = &mut self.ctx.frames[self.ctx.fp - 1];
                    push!(ret);
                }
                Op::iastore => astore!(i64),
                Op::uastore => astore!(u64),
                Op::unit => push!(Val::Unit),
                Op::dastore => astore!(f64),
                Op::rastore => {}
                Op::raload => {}
                Op::rloadl => {}
                Op::rstorel => {}
                Op::istorel | Op::ustorel | Op::dstorel => {
                    let val = self.ctx.stack[self.bp + frame!().sp - 1];
                    let index = frame_mut!().read_byte();
                    self.ctx.stack[index as usize] = val;
                }
                Op::iloadl | Op::uloadl | Op::dloadl => {
                    let index = frame_mut!().read_byte();
                    push!(self.ctx.stack[index as usize])
                }
                Op::iaload | Op::uaload | Op::daload => {
                    let index = pop!().as_prm() as isize;
                    let array_ref = pop!();
                    push!(array_ref.as_array().get::<u64>(index));
                }
                Op::newarr => {
                    let len = pop!().as_prm() as usize;
                    let ty = frame_mut!().read_type()?;
                    let array = Array::new(len, ty);
                    push!(self.heap.alloc_and_gc(array, &self.ctx));
                }
                Op::invoke => {
                    // ... <f> <arg0> ... <argn> <stack_top>
                    let argc = frame_mut!().read_byte() as usize;
                    // index of the function pointer
                    let f_idx = self.bp + frame!().sp - argc - 1;
                    let f = self.ctx.stack[f_idx].as_fn();
                    self.ctx.frames[self.ctx.fp] = Frame::new(f, self.bp);
                    frame = &mut self.ctx.frames[self.ctx.fp];
                    self.ctx.fp += 1;
                    // set base pointer to the function of the frame
                    self.bp = f_idx;
                }
                Op::ldc => {
                    let index = frame_mut!().read_byte() as usize;
                    let constant = self.ctx.constants[index];
                    push!(constant)
                }
            }
        })
    }
}

/// note the two first allocations are for the start function and the main function
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() -> VMResult<()> {
        let main_code = CodeBuilder::default()
            .emit_iconst(5)
            .emit_op(Op::ret)
            .build();
        let executable = Executable::from(Function::new(main_code));
        let mut vm = VM::new(executable);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Prm(5));
        Ok(())
    }

    /// tests the vm applies arithmetic left to right
    #[test]
    fn vm_arith_order() -> VMResult<()> {
        let code = CodeBuilder::default()
            .emit_op(Op::iconst)
            .emit_const(7i64)
            .emit_op(Op::iconst)
            .emit_const(5i64)
            .emit_op(Op::isub)
            .emit_op(Op::iret)
            .build();
        let executable = Executable::from(Function::new(code));
        let mut vm = VM::new(executable);
        let ret = vm.run()?;
        assert_eq!(ret, Val::Prm(2));
        Ok(())
    }

    #[test]
    fn vm_array() -> VMResult<()> {
        let code = CodeBuilder::default()
            .emit_array(Type::U, 4)
            .emit_op(Op::ret)
            .build();
        let mut vm = VM::new(Function::new(code).into());
        let _ret = vm.run()?;
        // dbg!(vm.gc);
        // assert_eq!(ret, Val::Prim(2));
        Ok(())
    }

    #[test]
    fn vm_array_load_store() -> VMResult<()> {
        // let xs = [0u8; 4];
        // xs[3] = 5;
        // xs[1] = 2;
        // xs[3] - xs[1]
        let code = CodeBuilder::default()
            .emit_array(Type::U, 4)
            .emit_loadl(0)
            .emit_iastore(3, 5)
            .emit_loadl(1)
            .emit_iastore(1, 2)
            .emit_loadl(0)
            .emit_iaload(3)
            .emit_loadl(0)
            .emit_iaload(1)
            .emit_op(Op::isub)
            .emit_op(Op::iret)
            .build();

        let mut vm = VM::new(Function::new(code).into());
        let ret = vm.run()?;
        assert_eq!(ret, Val::Prm(3));
        Ok(())
    }

    #[test]
    #[cfg(debug_assertions)]
    fn gc_release_unused_array() -> VMResult<()> {
        let code = CodeBuilder::default()
            .emit_array(Type::U, 4)
            .emit_iastore(0, 8)
            // when the second array is allocated the first should be freed as there are no
            // references to it
            .emit_array(Type::U, 8)
            .emit_op(Op::ret)
            .build();
        let mut vm = VM::new(Function::new(code).into());
        vm.run()?;
        // assert that the first array that was allocated is now freed
        assert!(vm.heap.gc.dbg_allocations[2].is_none());
        // assert that the first thing that was allocated is NOT freed
        assert!(vm.heap.gc.dbg_allocations[3].is_some());
        Ok(())
    }

    #[test]
    #[cfg(debug_assertions)]
    fn gc_maintain_arrays() -> VMResult<()> {
        let code = CodeBuilder::default()
            .emit_array(Type::U, 4)
            .emit_array(Type::U, 8)
            .emit_op(Op::ret)
            .build();
        let mut vm = VM::new(Function::new(code).into());
        vm.run()?;
        // println!("{:?}", vm.heap.gc);
        assert!(vm.heap.gc.dbg_allocations[0].is_some());
        assert!(vm.heap.gc.dbg_allocations[1].is_some());
        Ok(())
    }

    #[test]
    #[cfg(debug_assertions)]
    fn gc_release_multiple_unused_arrays() -> VMResult<()> {
        let code = CodeBuilder::default()
            .emit_array(Type::U, 4)
            .emit_array(Type::U, 8)
            .emit_iastore(0, 3)
            .emit_array(Type::U, 8)
            .emit_iastore(0, 5)
            .emit_array(Type::U, 8)
            .emit_op(Op::ret)
            .build();

        let mut vm = VM::new(Function::new(code).into());
        vm.run()?;
        assert!(vm.heap.gc.dbg_allocations[2].is_some());
        assert!(vm.heap.gc.dbg_allocations[3].is_none());
        assert!(vm.heap.gc.dbg_allocations[4].is_none());
        assert!(vm.heap.gc.dbg_allocations[5].is_some());
        Ok(())
    }
}

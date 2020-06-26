mod ctx;
mod gc_tests;

use super::*;
use crate::compiler::{Constant, Executable};
use crate::error::*;
use crate::gc::{GarbageCollector, Gc, Trace, GC};
use ctx::Ctx;

#[derive(Default, Debug)]
pub struct Heap<G> {
    gc: G,
}

impl<G> Heap<G>
where
    G: GarbageCollector,
{
    pub fn new(gc: G) -> Self {
        Self { gc }
    }

    pub fn alloc_and_gc<T>(&mut self, t: T, root: impl Trace) -> Gc<T>
    where
        T: Trace + 'static,
    {
        self.gc.mark_sweep(root);
        self.gc.alloc(t)
    }
}

/// the virtual machine
pub struct VM<G> {
    /// base pointer; points to where in the stack the current frame starts (i.e. the index of the
    /// currently executing function ptr)
    bp: usize,
    ctx: Ctx,
    heap: Heap<G>,
}

impl VM<GC> {
    pub fn with_default_gc(executable: Executable) -> Self {
        Self::new(GC::default(), executable)
    }
}

impl<G> VM<G>
where
    G: GarbageCollector,
{
    pub fn new(gc: G, executable: Executable) -> Self {
        let mut heap = Heap::new(gc);

        let Executable { constants, start } = executable;
        let f = heap.gc.alloc(start);
        // allocate all the constants once upfront
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

        macro_rules! read_const {
            () => {{
                let index = frame_mut!().read_byte() as usize;
                self.ctx.constants[index]
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
                    let array = self.alloc(Array::new(len, ty));
                    push!(array)
                }
                Op::clsr => {
                    let f = read_const!().as_fn();
                    let clsr = self.alloc(Closure::new(f.ptr));
                    push!(clsr);
                }
                Op::invoke => {
                    // ... <f> <arg0> ... <argn> <stack_top>
                    let argc = frame_mut!().read_byte() as usize;
                    // index of the function pointer
                    let f_idx = self.bp + frame!().sp - argc - 1;
                    let f = self.ctx.stack[f_idx];
                    match f {
                        Val::Fn(f) => {
                            self.ctx.frames[self.ctx.fp] = Frame::new(f, self.bp);
                            frame = &mut self.ctx.frames[self.ctx.fp];
                            self.ctx.fp += 1;
                            // set base pointer to the function of the frame
                            self.bp = f_idx;
                        }
                        Val::Clsr(f) => todo!(),
                        x => panic!("expected invokable, found `{:?}`", x),
                    }
                }
                Op::ldc => push!(read_const!()),
            }
        })
    }

    fn alloc<T>(&mut self, t: T) -> Gc<T>
    where
        T: Trace + 'static,
    {
        self.heap.alloc_and_gc(t, &self.ctx)
    }
}

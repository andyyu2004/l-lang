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
        let clsr = heap.gc.alloc(Closure::new(f));
        // allocate all the constants once upfront
        let constants = constants
            .into_iter()
            .map(|c| match c {
                Constant::Function(f) => Val::Fn(heap.gc.alloc(f)),
                Constant::String(s) => Val::Str(heap.gc.alloc(s)),
            })
            .collect();

        Self {
            heap,
            ctx: Ctx::new(clsr, constants),
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
                self.ctx.stack[self.ctx.bp + frame!().sp] = $value.into();
                frame_mut!().sp += 1;
            }};
        }

        macro_rules! pop {
            () => {{
                frame_mut!().sp -= 1;
                self.ctx.stack[self.ctx.bp + frame!().sp]
            }};
        }

        macro_rules! iarith {
            ($op: tt) => {{
                let r = pop!().as_i64();
                let l = pop!().as_i64();
                push!(l $op r);
            }}
        }

        macro_rules! uarith {
            ($op: tt) => {{
                let r = pop!().as_u64();
                let l = pop!().as_u64();
                push!(l $op r);
            }}
        }

        macro_rules! farith {
            ($op: tt) => {{
                let r = pop!().as_f64();
                let l = pop!().as_f64();
                push!(l $op r);
            }}
        }

        macro_rules! astore {
            ($value:expr) => {{
                let index = pop!().as_u64() as isize;
                let array_ref = pop!();
                array_ref.as_array().set(index, $value);
            }};
        }

        /// load constant from constant pool
        macro_rules! load_const {
            () => {{
                let index = frame_mut!().read_byte() as usize;
                self.ctx.constants[index]
            }};
        }

        /// read constant from code
        macro_rules! read_const {
            ($ty:ty) => {
                push!(frame_mut!().read_u64() as $ty)
            };
        }

        macro_rules! aload {
            ($ty:ty) => {{
                let index = pop!().as_u64() as isize;
                let array_ref = pop!();
                push!(array_ref.as_array().get::<$ty>(index));
            }};
        }

        Ok(loop {
            // println!("{:?}", &self.ctx.stack[..frame!().sp]);
            match frame_mut!().read_opcode()? {
                Op::nop => {}
                Op::iconst => read_const!(i64),
                Op::uconst => read_const!(u64),
                Op::dconst => push!(f64::from_bits(frame_mut!().read_u64())),
                Op::iadd => iarith!(+),
                Op::uadd => uarith!(+),
                Op::dadd => farith!(+),
                Op::isub => iarith!(-),
                Op::usub => uarith!(-),
                Op::dsub => farith!(-),
                Op::imul => iarith!(*),
                Op::umul => uarith!(*),
                Op::dmul => farith!(*),
                Op::idiv => iarith!(/),
                Op::udiv => uarith!(/),
                Op::ddiv => farith!(/),
                Op::pop => frame_mut!().sp -= 1,
                Op::ret | Op::uret | Op::dret | Op::iret => {
                    let ret = pop!();
                    self.ctx.fp -= 1;
                    if self.ctx.fp == 0 {
                        break ret;
                    }
                    self.ctx.bp = frame!().ret_addr;
                    frame = &mut self.ctx.frames[self.ctx.fp - 1];
                    push!(ret);
                }
                Op::iastore => {
                    // this can't be inlined as macro expansion is lazy and we must pop the value first
                    let value = pop!().as_i64();
                    astore!(value);
                }
                Op::uastore => {
                    let value = pop!().as_u64();
                    astore!(value)
                }
                Op::dastore => {
                    let value = pop!().as_f64();
                    astore!(value)
                }
                Op::unit => push!(Val::Unit),
                Op::rastore => {}
                Op::raload => {}
                Op::rloadl => {}
                Op::rstorel => {}
                Op::istorel | Op::ustorel | Op::dstorel => {
                    let val = self.ctx.stack[self.ctx.bp + frame!().sp - 1];
                    let index = frame_mut!().read_byte();
                    self.ctx.stack[index as usize] = val;
                }
                Op::iloadl | Op::uloadl | Op::dloadl => {
                    let index = frame_mut!().read_byte();
                    push!(self.ctx.stack[index as usize])
                }
                Op::iaload => aload!(i64),
                Op::uaload => aload!(u64),
                Op::daload => aload!(f64),
                Op::newarr => {
                    let len = pop!().as_u64() as usize;
                    let ty = frame_mut!().read_type()?;
                    let array = self.alloc(Array::new(len, ty));
                    push!(array)
                }
                Op::clsr => {
                    let f = load_const!().as_fn();
                    let clsr = self.alloc(Closure::new(f));
                    push!(clsr);
                }
                Op::invoke => {
                    // ... <f> <arg0> ... <argn> <stack_top>
                    let argc = frame_mut!().read_byte() as usize;
                    // index of the function pointer
                    let f_idx = self.ctx.bp + frame!().sp - argc - 1;
                    let f = self.ctx.stack[f_idx];
                    let clsr = match f {
                        Val::Fn(f) => self.alloc(Closure::new(f)),
                        Val::Clsr(clsr) => clsr,
                        x => panic!("expected invokable, found `{:?}`", x),
                    };
                    self.ctx.frames[self.ctx.fp] = Frame::new(clsr, self.ctx.bp);
                    frame = &mut self.ctx.frames[self.ctx.fp];
                    self.ctx.fp += 1;
                    // set base pointer to the function of the frame
                    self.ctx.bp = f_idx;

                    // for i in 0..clsr.f.upvalc as usize {
                    //     let in_enclosing = frame_mut!().read_byte();
                    //     let index = frame_mut!().read_byte() as usize;
                    //     clsr.upvals[i] = if in_enclosing == 0 { todo!() } else { todo!() };
                    // }
                }
                Op::ldc => push!(load_const!()),
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

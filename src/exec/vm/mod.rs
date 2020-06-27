mod closure_tests;
mod ctx;
mod function_tests;
mod gc_tests;
mod var_tests;

use super::*;
use crate::compiler::{Constant, Executable};
use crate::error::*;
use crate::gc::{GarbageCollector, Gc, Trace, GC};
use ctx::Ctx;
use std::{cell::Cell, ptr::NonNull};

#[derive(Default, Debug)]
pub struct Heap<G> {
    gc: G,
    disabled: Cell<bool>,
}

impl<G> Heap<G>
where
    G: GarbageCollector,
{
    pub fn new(gc: G) -> Self {
        Self {
            gc,
            disabled: Cell::new(false),
        }
    }

    pub fn disable_gc(&self) {
        self.disabled.set(true)
    }

    pub fn enable_gc(&self) {
        self.disabled.set(false)
    }

    pub fn alloc_and_gc<T>(&mut self, t: T, root: impl Trace) -> Gc<T>
    where
        T: Trace + 'static,
    {
        if !self.disabled.get() {
            self.gc.mark_sweep(root);
        }
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

        macro_rules! read_byte {
            () => {
                frame_mut!().read_byte()
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
                let index = read_byte!() as usize;
                self.ctx.constants[index]
            }};
        }

        /// read constant from code
        macro_rules! read_const {
            ($ty:ty) => {
                frame_mut!().read_u64() as $ty
            };
        }

        macro_rules! aload {
            ($ty:ty) => {{
                let index = pop!().as_u64() as isize;
                let array_ref = pop!();
                push!(array_ref.as_array().get::<$ty>(index));
            }};
        }

        macro_rules! peek {
            ($i:expr) => {
                self.ctx.stack[self.ctx.bp + frame!().sp - $i - 1];
            };
        }

        Ok(loop {
            let opcode = frame_mut!().read_opcode()?;
            println!(
                "op:{:?} bp:{} sp:{} {:?}",
                opcode,
                self.ctx.bp,
                frame!().sp,
                &self.ctx.stack[..self.ctx.bp + frame!().sp]
            );
            match opcode {
                Op::nop => panic!("no-op"),
                Op::iconst => push!(read_const!(i64)),
                Op::uconst => push!(read_const!(u64)),
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
                Op::ret | Op::uret | Op::dret | Op::iret | Op::rret => {
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
                    self.ctx.stack[self.ctx.bp + read_byte!() as usize] = peek!(0)
                }
                Op::iloadl | Op::uloadl | Op::dloadl => {
                    push!(self.ctx.stack[self.ctx.bp + read_byte!() as usize])
                }
                Op::iaload => aload!(i64),
                Op::uaload => aload!(u64),
                Op::daload => aload!(f64),
                Op::newarr => {
                    let len = pop!().as_u64() as usize;
                    let ty = frame_mut!().read_type()?;
                    push!(self.alloc(Array::new(len, ty)))
                }
                Op::uloadu => {}
                Op::dloadu => {}
                Op::rloadu => {}
                Op::iloadu => {
                    let i = read_byte!() as usize;
                    let upval = &*frame!().clsr.upvals[i];
                    let val = **upval;
                    push!(val)
                }
                Op::istoreu => {
                    let val = peek!(0);
                    let i = read_byte!() as usize;
                    **frame_mut!().clsr.upvals[i] = val;
                }
                Op::ustoreu => todo!(),
                Op::dstoreu => todo!(),
                Op::rstoreu => todo!(),
                Op::ldc => push!(load_const!()),
                Op::clsr => {
                    let f = load_const!().as_fn();
                    // we may be allocating unrooted upvalues in this section so we must disable gc
                    let clsr = self.without_gc(|this| {
                        let mut clsr = this.alloc(Closure::new(f));
                        for i in 0..clsr.f.upvalc as usize {
                            let in_enclosing = read_byte!();
                            let index = read_byte!() as usize;
                            let upvalue = if in_enclosing == 0 {
                                // if the upvalue is not in the directly enclosing scope, get the
                                // upvalue from the enclosing closure
                                frame!().clsr.upvals[index]
                            } else {
                                // if the upvalue closes over a variable in the immediately enclosing function, capture it
                                this.capture_upval(index)
                            };
                            assert_eq!(clsr.upvals.len(), i);
                            clsr.upvals.push(upvalue);
                        }
                        clsr
                    });
                    push!(clsr);
                }
                Op::clsupv => self.close_upvalue(read_byte!()),
                Op::invoke => {
                    // ... <f> <arg0> ... <argn> <stack_top>
                    let argc = read_byte!() as usize;
                    // index of the function pointer
                    let f_idx = self.ctx.bp + frame!().sp - argc - 1;
                    let f = self.ctx.stack[f_idx];
                    let clsr = match f {
                        Val::Fn(f) => {
                            // if f is just a function not a closure, it shouldn't have any upvalues
                            assert!(f.upvalc == 0);
                            self.alloc(Closure::new(f))
                        }
                        Val::Clsr(clsr) => clsr,
                        x => panic!("expected invokable, found `{:?}`", x),
                    };
                    frame_mut!().sp -= 1 + argc;
                    self.ctx.frames[self.ctx.fp] = Frame::new(clsr, self.ctx.bp);
                    frame = &mut self.ctx.frames[self.ctx.fp];
                    frame_mut!().sp = argc;
                    self.ctx.fp += 1;
                    // set base pointer to the slot above the function of the frame (so locals are
                    // indexed from 0)
                    self.ctx.bp = f_idx + 1;
                }
            }
        })
    }

    fn without_gc<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.heap.disable_gc();
        let t = f(self);
        self.heap.enable_gc();
        t
    }

    /// moves the upvalue that points at the value at `index` onto the heap
    fn close_upvalue(&mut self, index: u8) {
        let val_ref = &mut self.ctx.stack[self.ctx.bp + index as usize];
        let val_ptr = NonNull::new(val_ref).unwrap();
        let mut upval_ptr = self.ctx.open_upvalues.remove(&val_ptr).unwrap();
        // assert that the upvalue actually points at the value we are moving onto the heap
        debug_assert_eq!(val_ref, &**upval_ptr);
        // allocate the value and mutate the open upvalue to a closed upvalue
        *upval_ptr = Upval::Closed(self.alloc(**upval_ptr));
    }

    /// captures the value at `index` in an upvalue
    fn capture_upval(&mut self, index: usize) -> Gc<Upval> {
        let val_ptr = NonNull::new(&mut self.ctx.stack[self.ctx.bp + index]).unwrap();
        match self.ctx.open_upvalues.get(&val_ptr) {
            Some(&upvalue) => upvalue,
            None => {
                let upval = Upval::Open(val_ptr);
                let upval_gc_ptr = self.alloc(upval);
                self.ctx.open_upvalues.insert(val_ptr, upval_gc_ptr);
                upval_gc_ptr
            }
        }
    }

    fn alloc<T>(&mut self, t: T) -> Gc<T>
    where
        T: Trace + 'static,
    {
        self.heap.alloc_and_gc(t, &self.ctx)
    }
}

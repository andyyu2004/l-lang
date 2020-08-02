use super::*;
use crate::compiler::{Constant, Executable};
use crate::error::*;
use crate::exec::*;
use crate::gc::{GarbageCollector, Gc, Trace, GC};
use std::{cell::Cell, ptr::NonNull};

/// the virtual machine
pub struct VM<G> {
    pub(super) ctx: VMCtx,
    pub(super) heap: Heap<G>,
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
        // the odd mapping of `Const::Fn -> Val::Closure` and `Const::Lambda -> Val::Fn`
        // is because `Fn` cannot capture upvars so we allocate it as a closure upfront for efficiency
        // `Lambda` may capture upvars so we only allocate it is a `Fn` so the `VM` allocates a new
        // closure for it each time it is used
        let constants = constants
            .into_iter()
            .map(|c| match c {
                Constant::Function(f) => Val::Clsr(heap.mk_clsr(f)),
                Constant::String(s) => Val::Str(heap.gc.alloc(s)),
                Constant::Lambda(f) => Val::Fn(heap.gc.alloc(f)),
            })
            .collect();

        Self { heap, ctx: VMCtx::new(clsr, constants) }
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

        macro_rules! dbg_stack {
            () => {
                println!("{:?}", &self.ctx.stack[..self.ctx.bp + frame!().sp])
            };
        }

        macro_rules! read16 {
            () => {{
                let x = read_byte!() as u16;
                let y = read_byte!() as u16;
                x << 8 | y
            }};
        }

        /// offsets a usize by an isize
        macro_rules! offset {
            ($u:expr, $i:expr) => {{
                if $i < 0 {
                    $u -= $i as usize;
                } else {
                    $u += $i as usize;
                }
            }};
        }

        /// jmps if the given predicate `p` is true
        macro_rules! jmp {
            ($p:expr) => {{
                let offset = read16!() as isize;
                if $p {
                    offset!(frame_mut!().ip, offset)
                }
            }};
        }

        /// returns index of the top of stack
        macro_rules! top {
            () => {{ self.ctx.bp + frame!().sp }};
        }

        Ok(loop {
            let opcode = frame_mut!().read_opcode()?;
            // println!(
            //     "op:{:?} bp:{} sp:{} {:#?}",
            //     opcode,
            //     self.ctx.bp,
            //     frame!().sp,
            //     &self.ctx.stack[..self.ctx.bp + frame!().sp]
            // );
            match opcode {
                Op::nop => panic!("no-op"),
                Op::dup => push!(peek!(0)),
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
                Op::dcmplt => farith!(<),
                Op::dcmpgt => farith!(>),
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
                Op::rstorel => {}
                Op::istorel | Op::ustorel | Op::dstorel =>
                    self.ctx.stack[self.ctx.bp + read_byte!() as usize] = peek!(0),
                Op::iloadl | Op::uloadl | Op::dloadl | Op::rloadl =>
                    push!(self.ctx.stack[self.ctx.bp + read_byte!() as usize]),
                Op::iaload => aload!(i64),
                Op::uaload => aload!(u64),
                Op::daload => aload!(f64),
                Op::newarr => {
                    let len = pop!().as_u64() as usize;
                    let ty = frame_mut!().read_type()?;
                    push!(self.alloc(Array::new(len, ty)))
                }
                Op::uloadu | Op::dloadu | Op::rloadu | Op::iloadu => {
                    let i = read_byte!() as usize;
                    let upvar = &*frame!().clsr.upvars[i];
                    let val = **upvar;
                    push!(val)
                }
                Op::istoreu | Op::ustoreu | Op::dstoreu | Op::rstoreu => {
                    let val = peek!(0);
                    let i = read_byte!() as usize;
                    **frame_mut!().clsr.upvars[i] = val;
                }
                Op::ldc => push!(load_const!()),
                Op::mkclsr => {
                    // let f = load_const!().as_fn();
                    let f = match load_const!() {
                        Val::Fn(f) => f,
                        Val::Clsr(s) => panic!("found closure when making closure"),
                        _ => panic!(),
                    };
                    // we may be allocating unrooted upvars in this section so we must disable gc
                    let clsr = self.without_gc(|vm| {
                        let mut clsr = vm.alloc(Closure::new(f));
                        let upvarc = read_byte!();
                        for i in 0..upvarc as usize {
                            let in_enclosing = read_byte!();
                            let index = read_byte!() as usize;
                            let upvalue = if in_enclosing == 0 {
                                // if the upvar is not in the directly enclosing scope, get the
                                // upvar from the enclosing closure
                                frame!().clsr.upvars[index]
                            } else {
                                // if the upvar closes over a variable in the immediately enclosing function, capture it directly
                                vm.capture_upval(index)
                            };
                            assert_eq!(clsr.upvars.len(), i);
                            clsr.upvars.push(upvalue);
                        }
                        clsr
                    });
                    push!(clsr)
                }
                Op::call => {
                    // ... <f> <arg0> ... <argn> <stack_top>
                    let argc = read_byte!() as usize;
                    // index of the function pointer
                    let f_idx = top!() - argc - 1;
                    let f = self.ctx.stack[f_idx];
                    let clsr = match f {
                        Val::Data(d) => {
                            push!(self.alloc(Instance::new(d)));
                            continue;
                        }
                        Val::Clsr(clsr) => clsr,
                        Val::Fn(_) => panic!("fn should be wrapped in closure before invocation"),
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
                Op::popscp => {
                    let n = read_byte!() as usize;
                    let val = pop!();
                    frame_mut!().sp -= n;
                    // close over all the upvars that may have just been popped off
                    self.close_upvars(top!());
                    push!(val);
                }
                Op::mktup => {
                    let n = read_byte!() as usize;
                    let elements = Vec::from(&self.ctx.stack[top!() - n..top!()]);
                    let tup = self.alloc(Tuple::new(elements));
                    // make sure to not decrease the stack pointer before allocating (coz gc)
                    frame_mut!().sp -= n;
                    push!(tup);
                }
                Op::mklst => {}
                Op::mkmap => {}
                Op::jmp => jmp!(true),
                Op::jmpt => jmp!(pop!().as_u64() != 0),
                Op::jmpf => jmp!(pop!().as_u64() == 0),
                Op::jmpeq => jmp!(pop!() == pop!()),
                Op::jmpneq => jmp!(pop!() != pop!()),
            }
        })
    }

    fn without_gc<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.heap.disable_gc();
        let t = f(self);
        self.heap.enable_gc();
        t
    }

    /// moves the open upvalues that points at the value at `index` or above onto the heap
    fn close_upvars(&mut self, index: usize) {
        self.without_gc(|vm| {
            let val_ref = &mut vm.ctx.stack[index];
            let val_ptr = NonNull::new(val_ref).unwrap();

            // this vector contains all the open upvalues that are above the given stack index
            let to_close: Vec<Gc<Upvar>> = vm
                .ctx
                .open_upvars
                .drain_filter(|&ptr, _| ptr >= val_ptr)
                .map(|(_, upvar_ptr)| upvar_ptr)
                .collect();

            debug_assert!(to_close.iter().all(|upvar| upvar.is_open()));

            // allocate the value pointed to by the upvalue,
            // then mutate the open upvalue to a closed upvalue
            for mut upvar_ptr in to_close {
                let val: Val = **upvar_ptr;
                *upvar_ptr = Upvar::Closed(vm.alloc(val));
            }
        });
    }

    /// captures the value at `index` in an upvalue
    fn capture_upval(&mut self, index: usize) -> Gc<Upvar> {
        let val_ptr = NonNull::new(&mut self.ctx.stack[self.ctx.bp + index]).unwrap();
        // if the upvalue already exists, reuse it, otherwise, allocate a new one
        match self.ctx.open_upvars.get(&val_ptr) {
            Some(&upvar) => upvar,
            None => {
                let upvar = Upvar::Open(val_ptr);
                let upvar_ptr = self.alloc(upvar);
                self.ctx.open_upvars.insert(val_ptr, upvar_ptr);
                upvar_ptr
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

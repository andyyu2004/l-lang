//! compile bytecode into LLVM
//! this bytecode is not suitable for lowering into LLVM IR as too much type information has been
//! lost at this point
//! it is non-trivial to add a 'tcx lifetime into this stuff due to the GC

use super::CodegenCtx;
use crate::compiler::{ConstId, Constant, Executable};
use crate::exec::vm::STACK_MAX;
use crate::exec::{Function, Op};
use indexed_vec::Idx;
use inkwell::values::{AnyValueEnum, FunctionValue};
use std::convert::{TryFrom, TryInto};

impl<'tcx> CodegenCtx<'tcx> {
    pub fn compile_bytecode(&mut self, executable: Executable) {
        let mut sp = 0;
        let mut stack: [AnyValueEnum; STACK_MAX] =
            [self.ctx.i64_type().const_zero().into(); STACK_MAX];

        let mut code = executable.start.code.as_slice();
        let constants = executable.constants;

        macro_rules! pop {
            () => {{
                sp -= 1;
                stack[sp]
            }};
        }

        macro_rules! dpop {
            () => {{ pop!().into_float_value() }};
        }

        macro_rules! read8 {
            () => {{
                let xs: [u8; 8] = code[1..9].try_into().unwrap();
                xs
            }};
        }

        macro_rules! read_u64 {
            () => {{ u64::from_le_bytes(read8!()) }};
        }

        macro_rules! ipop {
            () => {{ pop!().into_int_value() }};
        }

        macro_rules! push {
            ($val:expr) => {{ stack[sp] = $val.into() }};
        }

        macro_rules! ibin {
            ($method:ident, $name:expr) => {{ push!(self.builder.$method(ipop!(), ipop!(), $name)) }};
        }

        macro_rules! dbin {
            ($method:ident, $name:expr) => {{ push!(self.builder.$method(dpop!(), dpop!(), $name)) }};
        }

        while code.len() > 0 {
            let op = Op::try_from(code[0]).unwrap();
            match op {
                Op::nop => todo!(),
                Op::iconst => push!(self.ctx.i64_type().const_int(read_u64!(), false)),
                Op::uconst => todo!(),
                Op::dconst => push!(self.ctx.f64_type().const_float(f64::from_bits(read_u64!()))),
                Op::iadd => ibin!(build_int_add, "iadd"),
                Op::uadd => todo!(),
                Op::dadd => dbin!(build_float_add, "dadd"),
                Op::isub => ibin!(build_int_sub, "isub"),
                Op::usub => todo!(),
                Op::dsub => dbin!(build_float_sub, "dsub"),
                Op::imul => ibin!(build_int_sub, "imul"),
                Op::umul => todo!(),
                Op::dmul => todo!(),
                Op::idiv => ibin!(build_int_signed_div, "idiv"),
                Op::udiv => ibin!(build_int_unsigned_div, "udiv"),
                Op::ddiv => dbin!(build_float_div, "ddiv"),
                Op::dcmplt => todo!(),
                Op::dcmpgt => todo!(),
                Op::jmp => todo!(),
                Op::jmpf => todo!(),
                Op::jmpt => todo!(),
                Op::jmpeq => todo!(),
                Op::jmpneq => todo!(),
                Op::iret => todo!(),
                Op::uret => todo!(),
                Op::dret => todo!(),
                Op::rret => todo!(),
                Op::ret => todo!(),
                Op::unit => todo!(),
                Op::pop => todo!(),
                Op::dup => todo!(),
                Op::iloadl => todo!(),
                Op::uloadl => todo!(),
                Op::dloadl => todo!(),
                Op::rloadl => todo!(),
                Op::istorel => todo!(),
                Op::ustorel => todo!(),
                Op::dstorel => todo!(),
                Op::rstorel => todo!(),
                Op::iloadu => todo!(),
                Op::uloadu => todo!(),
                Op::dloadu => todo!(),
                Op::rloadu => todo!(),
                Op::istoreu => todo!(),
                Op::ustoreu => todo!(),
                Op::dstoreu => todo!(),
                Op::rstoreu => todo!(),
                Op::ldc => {
                    let idx = code[1] as usize;
                    let c = &constants[ConstId::new(idx)];
                    match c {
                        Constant::Function(f) => push!(self.compile_bytecode_function(f)),
                        Constant::Lambda(f) => todo!(),
                        Constant::String(s) => todo!(),
                    }
                }
                Op::newarr => todo!(),
                Op::iaload => todo!(),
                Op::uaload => todo!(),
                Op::daload => todo!(),
                Op::raload => todo!(),
                Op::iastore => todo!(),
                Op::uastore => todo!(),
                Op::dastore => todo!(),
                Op::rastore => todo!(),
                Op::call => todo!(),
                Op::mkclsr => todo!(),
                Op::popscp => todo!(),
                Op::mktup => todo!(),
                Op::mklst => todo!(),
                Op::mkmap => todo!(),
            };
            code = &code[op.size()..];
        }
    }

    fn compile_bytecode_function(&mut self, f: &Function) -> FunctionValue<'tcx> {
        todo!()
    }
}

use super::{Disassembler, Op};
use crate::{exec::Type, util::As8Bytes};
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Deref, DerefMut)]
pub struct Code(Vec<u8>);

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Ok(Disassembler::new(&self.0, f).fmt())
    }
}

/// convenience structure for hand building bytecode
#[derive(Default)]
pub struct CodeBuilder {
    code: Vec<u8>,
}

// we only take a &mut self as we use this in the compiler
// where we don't have convenient access to a owned builder
impl CodeBuilder {
    fn emit_byte(&mut self, byte: u8) -> &mut Self {
        self.code.push(byte);
        self
    }

    pub fn emit_op(&mut self, op: Op) -> &mut Self {
        self.emit_byte(op as u8)
    }

    pub fn emit_closure(&mut self, f_idx: u8, upvalues: Vec<(bool, u8)>) -> &mut Self {
        self.emit_op(Op::clsr).emit_byte(f_idx);
        for (in_enclosing, index) in upvalues {
            self.emit_upval(in_enclosing, index);
        }
        self
    }

    pub fn emit_close_upvalue(&mut self, index: u8) -> &mut Self {
        self.emit_op(Op::clsupv).emit_byte(index)
    }

    fn emit_upval(&mut self, in_enclosing: bool, index: u8) -> &mut Self {
        self.emit_byte(in_enclosing as u8).emit_byte(index)
    }

    pub fn emit_dconst(&mut self, d: f64) -> &mut Self {
        self.emit_op(Op::dconst).write_const(d)
    }

    pub fn emit_iconst(&mut self, i: i64) -> &mut Self {
        self.emit_op(Op::iconst).write_const(i)
    }

    pub fn emit_uconst(&mut self, u: u64) -> &mut Self {
        self.emit_op(Op::uconst).write_const(u)
    }

    /// writes a 8 byte constant into the code
    pub fn write_const(&mut self, c: impl As8Bytes) -> &mut Self {
        self.code.extend_from_slice(&c.as_bytes());
        self
    }

    pub fn emit_invoke(&mut self, argc: u8) -> &mut Self {
        self.emit_op(Op::invoke).emit_byte(argc)
    }

    pub fn emit_ldc(&mut self, idx: u8) -> &mut Self {
        self.emit_op(Op::ldc).emit_byte(idx)
    }

    pub fn emit_array(&mut self, ty: Type, size: u64) -> &mut Self {
        self.emit_uconst(size).emit_op(Op::newarr).emit_byte(ty as u8)
    }

    pub fn emit_iaload(&mut self, index: u64) -> &mut Self {
        self.emit_uconst(index as u64).emit_op(Op::iaload)
    }

    pub fn emit_loadl(&mut self, index: u8) -> &mut Self {
        self.emit_op(Op::iloadl).emit_byte(index)
    }

    pub fn emit_istorel(&mut self, index: u8, value: i64) -> &mut Self {
        self.emit_iconst(value).emit_op(Op::istorel).emit_byte(index)
    }

    pub fn emit_loadu(&mut self, index: u8) -> &mut Self {
        self.emit_op(Op::iloadu).emit_byte(index)
    }

    /// store a constant into an upvalue
    pub fn emit_istoreu_const(&mut self, index: u8, value: i64) -> &mut Self {
        self.emit_iconst(value).emit_storeu(index)
    }

    pub fn emit_storeu(&mut self, index: u8) -> &mut Self {
        self.emit_op(Op::istoreu).emit_byte(index)
    }

    pub fn emit_iastore(&mut self, index: u64, value: i64) -> &mut Self {
        self.emit_uconst(index).emit_iconst(value).emit_op(Op::iastore)
    }

    pub fn build(&mut self) -> Code {
        let code = std::mem::take(&mut self.code);
        Code(code)
    }
}

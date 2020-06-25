use super::Op;
use crate::{util::As8Bytes, Type};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Code(Vec<u8>);

#[derive(Default)]
pub struct CodeBuilder {
    code: Vec<u8>,
}

impl CodeBuilder {
    pub fn emit_op(self, op: Op) -> Self {
        self.emit_byte(op as u8)
    }

    fn emit_byte(mut self, byte: u8) -> Self {
        self.code.push(byte);
        self
    }

    pub fn emit_iconst(self, i: i64) -> Self {
        self.emit_op(Op::iconst).emit_const(i)
    }

    pub fn emit_uconst(self, u: u64) -> Self {
        self.emit_op(Op::uconst).emit_const(u)
    }

    /// writes a 8 byte constant into the code
    pub fn emit_const(mut self, c: impl As8Bytes) -> Self {
        self.code.extend_from_slice(&c.as_bytes());
        self
    }

    pub fn emit_invoke(self, argc: u8) -> Self {
        self.emit_op(Op::invoke).emit_byte(argc)
    }

    pub fn emit_ldc(self, idx: u8) -> Self {
        self.emit_op(Op::ldc).emit_byte(idx)
    }

    pub fn emit_array(self, ty: Type, size: u64) -> Self {
        self.emit_uconst(size)
            .emit_op(Op::newarr)
            .emit_byte(ty as u8)
    }

    pub fn emit_iaload(self, index: isize) -> Self {
        self.emit_iconst(index as i64).emit_op(Op::iaload)
    }

    pub fn emit_loadl(self, index: u64) -> Self {
        self.emit_op(Op::iloadl).emit_const(index)
    }

    pub fn emit_iastore(self, index: isize, value: i64) -> Self {
        self.emit_iconst(index as i64)
            .emit_iconst(value)
            .emit_op(Op::iastore)
    }

    pub fn build(self) -> Code {
        Code(self.code)
    }
}

impl Deref for Code {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Code {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

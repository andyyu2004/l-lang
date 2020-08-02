use super::{Disassembler, Op};
use crate::{exec::Type, util::As8Bytes};
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Deref, DerefMut, Clone, PartialEq, Eq)]
pub struct Code(Vec<u8>);

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Disassembler::new(&self, f).fmt()
    }
}

impl Debug for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
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
    pub fn code_len(&self) -> usize {
        self.code.len()
    }

    fn emit_byte(&mut self, byte: u8) -> &mut Self {
        self.code.push(byte);
        self
    }

    fn split_bytes(&mut self, bytes: u16) -> [u8; 2] {
        [(bytes >> 8 & 0xff) as u8, (bytes & 0xff) as u8]
    }

    // just for testing convenience, not used by the compiler
    pub fn emit_known_jmp(&mut self, jmp_op: Op, offset: u16) -> &mut Self {
        self.emit_op(jmp_op).emit16(offset)
    }

    pub fn emit16(&mut self, bytes: u16) -> &mut Self {
        let [a, b] = self.split_bytes(bytes);
        self.emit_byte(a);
        self.emit_byte(b)
    }

    /// given the offset and the first (0th) byte of the jmp instruction, patches the offset
    /// this function will subtract 3 off the offset for you to ignore the opcode and the
    /// two offset bytes of the jmp instruction
    pub fn patch_jmp(&mut self, jmp_start: usize, offset: u16) -> &mut Self {
        let [a, b] = self.split_bytes(offset - 3);
        self.code[jmp_start + 1] = a;
        self.code[jmp_start + 2] = b;
        self
    }

    pub fn emit_op(&mut self, op: Op) -> &mut Self {
        self.emit_byte(op as u8)
    }

    /// emits instruction for creating a `n`-tuple
    pub fn emit_tuple(&mut self, n: u8) -> &mut Self {
        self.emit_op(Op::mktup).emit_byte(n)
    }

    pub fn emit_closure(&mut self, f_idx: u8, upvars: Vec<(bool, u8)>) -> &mut Self {
        self.emit_op(Op::mkclsr).emit_byte(f_idx).emit_byte(upvars.len() as u8);
        for (in_enclosing, index) in upvars {
            self.emit_upvar(in_enclosing, index);
        }
        self
    }

    fn emit_upvar(&mut self, in_enclosing: bool, index: u8) -> &mut Self {
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
        self.emit_op(Op::call).emit_byte(argc)
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
        self.emit_op(Op::dloadl).emit_byte(index)
    }

    pub fn emit_istorel(&mut self, index: u8, value: i64) -> &mut Self {
        self.emit_iconst(value).emit_op(Op::istorel).emit_byte(index)
    }

    pub fn emit_popscp(&mut self, n: u8) -> &mut Self {
        self.emit_op(Op::popscp).emit_byte(n)
    }

    pub fn emit_loadu(&mut self, index: u8) -> &mut Self {
        self.emit_op(Op::dloadu).emit_byte(index)
    }

    /// store a constant into an upvar
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

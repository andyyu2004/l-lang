use super::Op;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter};

pub struct Disassembler<'a, 'f> {
    code: &'a [u8],
    f: &'a mut Formatter<'f>,
}

impl<'a, 'f> Disassembler<'a, 'f> {
    pub fn new(code: &'a [u8], f: &'a mut Formatter<'f>) -> Self {
        Self { code, f }
    }

    pub fn fmt(&mut self) {
        while !self.code.is_empty() {
            let len = self.fmt_inst();
            writeln!(self.f, "");
            self.code = &self.code[len..];
        }
    }

    fn simple_inst(&mut self) -> usize {
        self.write_op();
        1
    }

    /// writes the current opcode into the formatter
    fn write_op(&mut self) {
        write!(self.f, "{:?} ", self.op());
    }

    /// returns current opcode
    fn op(&self) -> Op {
        Op::try_from(self.code[0]).unwrap()
    }

    fn const_inst(&mut self) -> usize {
        self.write_op();
        let bits = self.code[1..9].try_into().unwrap();
        match self.op() {
            Op::iconst => write!(self.f, "{}", i64::from_le_bytes(bits)),
            Op::uconst => write!(self.f, "{}", u64::from_le_bytes(bits)),
            Op::dconst => write!(self.f, "{}", f64::from_bits(u64::from_le_bytes(bits))),
            _ => unreachable!(),
        };
        9
    }

    /// pretty printing instructions of the form
    /// <inst> <count>
    fn count_inst(&mut self) -> usize {
        self.write_op();
        write!(self.f, "{}", self.code[1]);
        2
    }

    /// prints out one instruction at a time and returns the length of the instruction
    fn fmt_inst(&mut self) -> usize {
        let f = &mut self.f;
        match self.op() {
            Op::iconst => self.const_inst(),
            Op::uconst => self.const_inst(),
            Op::dconst => self.const_inst(),
            Op::nop
            | Op::iadd
            | Op::uadd
            | Op::dadd
            | Op::isub
            | Op::usub
            | Op::dsub
            | Op::imul
            | Op::umul
            | Op::dmul
            | Op::idiv
            | Op::udiv
            | Op::ddiv
            | Op::iret
            | Op::uret
            | Op::dret
            | Op::rret
            | Op::ret
            | Op::unit
            | Op::mkmap
            | Op::pop => self.simple_inst(),
            Op::iloadl | Op::uloadl | Op::dloadl | Op::mktup | Op::mklst => self.count_inst(),
            Op::mkmap => todo!(),
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
            Op::ldc => todo!(),
            Op::newarr => todo!(),
            Op::iaload => todo!(),
            Op::uaload => todo!(),
            Op::daload => todo!(),
            Op::raload => todo!(),
            Op::iastore => todo!(),
            Op::uastore => todo!(),
            Op::dastore => todo!(),
            Op::rastore => todo!(),
            Op::invoke => todo!(),
            Op::clsr => todo!(),
            Op::clsupv => todo!(),
        }
    }
}

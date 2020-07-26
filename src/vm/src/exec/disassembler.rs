use super::Op;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter};

pub struct Disassembler<'a, 'f> {
    code: &'a [u8],
    i: usize,
    f: &'a mut Formatter<'f>,
}

impl<'a, 'f> Disassembler<'a, 'f> {
    pub fn new(code: &'a [u8], f: &'a mut Formatter<'f>) -> Self {
        Self { code, f, i: 0 }
    }

    pub fn fmt(&mut self) -> fmt::Result {
        while !self.code.is_empty() {
            let len = self.fmt_inst();
            writeln!(self.f)?;
            self.code = &self.code[len..];
            self.i += len;
        }
        Ok(())
    }

    fn dis_simple_inst(&mut self) -> usize {
        self.write_op();
        1
    }

    /// writes the current opcode into the formatter
    fn write_op(&mut self) {
        write!(self.f, "{:#04x} {:?} ", self.i, self.op()).unwrap()
    }

    /// returns current opcode
    fn op(&self) -> Op {
        Op::try_from(self.code[0]).unwrap()
    }

    fn const_inst(&mut self) -> usize {
        self.write_op();
        let bits = self.code[1..9].try_into().unwrap();
        match self.op() {
            Op::iconst => write!(self.f, "{:#04x}", i64::from_le_bytes(bits)),
            Op::uconst => write!(self.f, "{:#04x}", u64::from_le_bytes(bits)),
            Op::dconst => write!(self.f, "{:?}", f64::from_bits(u64::from_le_bytes(bits))),
            _ => unreachable!(),
        }
        .unwrap();
        9
    }

    /// pretty printing instructions of the form
    /// <inst> <count>
    fn count_inst(&mut self) -> usize {
        self.write_op();
        write!(self.f, "{:#4x?}", self.code[1]).unwrap();
        2
    }

    fn jmp_inst(&mut self) -> usize {
        self.write_op();
        let x = self.code[1] as u16;
        let y = self.code[2] as u16;
        let offset = (x << 8 | y) as usize;
        write!(self.f, "{:#06x} (-> {:#0x})", offset, self.i + offset + 3).unwrap();
        3
    }

    /// prints out one instruction at a time and returns the length of the instruction
    fn fmt_inst(&mut self) -> usize {
        let f = &mut self.f;
        match self.op() {
            Op::iconst => self.const_inst(),
            Op::uconst => self.const_inst(),
            Op::dconst => self.const_inst(),
            Op::jmp | Op::jmpt | Op::jmpf | Op::jmpeq | Op::jmpneq => self.jmp_inst(),
            Op::nop
            | Op::iadd
            | Op::dcmplt
            | Op::dcmpgt
            | Op::uadd
            | Op::dadd
            | Op::isub
            | Op::usub
            | Op::dsub
            | Op::dup
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
            | Op::pop => self.dis_simple_inst(),
            Op::popscp
            | Op::ldc
            | Op::iloadl
            | Op::uloadl
            | Op::dloadl
            | Op::mktup
            | Op::mklst
            | Op::call => self.count_inst(),
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
            Op::newarr => todo!(),
            Op::iaload => todo!(),
            Op::uaload => todo!(),
            Op::daload => todo!(),
            Op::raload => todo!(),
            Op::iastore => todo!(),
            Op::uastore => todo!(),
            Op::dastore => todo!(),
            Op::rastore => todo!(),
            Op::clsr => todo!(),
            Op::clsupv => todo!(),
        }
    }
}

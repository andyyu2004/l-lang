use super::Op;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter};

type FmtResult<T> = Result<T, fmt::Error>;

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
            let len = self.fmt_inst()?;
            writeln!(self.f)?;
            self.code = &self.code[len..];
            self.i += len;
        }
        Ok(())
    }

    fn dis_simple_inst(&mut self) -> FmtResult<usize> {
        self.write_op()?;
        Ok(1)
    }

    /// writes the current opcode into the formatter
    fn write_op(&mut self) -> FmtResult<()> {
        write!(self.f, "{:#04x} {:?} ", self.i, self.op())
    }

    /// returns current opcode
    fn op(&self) -> Op {
        Op::try_from(self.code[0]).unwrap()
    }

    fn const_inst(&mut self) -> FmtResult<usize> {
        self.write_op()?;
        let bits = self.code[1..9].try_into().unwrap();
        match self.op() {
            Op::iconst => write!(self.f, "{:#04x}", i64::from_le_bytes(bits))?,
            Op::uconst => write!(self.f, "{:#04x}", u64::from_le_bytes(bits))?,
            Op::dconst => write!(self.f, "{:?}", f64::from_bits(u64::from_le_bytes(bits)))?,
            _ => unreachable!(),
        };
        Ok(self.op_size())
    }

    /// pretty printing instructions of the form
    /// <inst> <count>
    fn count_inst(&mut self) -> FmtResult<usize> {
        self.write_op()?;
        write!(self.f, "{:#4x?}", self.code[1])?;
        Ok(self.op_size())
    }

    fn op_size(&mut self) -> usize {
        self.op().size()
    }

    fn jmp_inst(&mut self) -> FmtResult<usize> {
        self.write_op()?;
        let x = self.code[1] as u16;
        let y = self.code[2] as u16;
        let offset = (x << 8 | y) as usize;
        write!(self.f, "{:#06x} (-> {:#0x})", offset, self.i + offset + 3)?;
        Ok(3)
    }

    fn fmt_clsr(&mut self) -> FmtResult<usize> {
        self.write_op()?;
        write!(self.f, "{:#4x}", self.code[1]).unwrap();
        let upvarc = self.code[2] as usize;
        for i in 0..upvarc {
            write!(self.f, "\n|\t").unwrap();
            let idx = self.code[3 + 2 * i + 1];
            if self.as_bool(self.code[3 + 2 * i]) {
                write!(self.f, "local {:#4x}", idx)?;
            } else {
                write!(self.f, "upvar {:#4x}", idx)?;
            }
        }
        Ok(3 + 2 * upvarc)
    }

    fn as_bool(&mut self, i: u8) -> bool {
        match i {
            0 => false,
            1 => true,
            _ => panic!("invalid bool `{}`", i),
        }
    }

    /// prints out one instruction at a time and returns the length of the instruction
    fn fmt_inst(&mut self) -> FmtResult<usize> {
        let f = &mut self.f;
        match self.op() {
            Op::iconst | Op::uconst | Op::dconst => self.const_inst(),
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
            | Op::istorel
            | Op::ustorel
            | Op::dstorel
            | Op::rstorel
            | Op::uloadl
            | Op::iloadu
            | Op::rloadl
            | Op::dloadl
            | Op::mktup
            | Op::mklst
            | Op::uloadu
            | Op::dloadu
            | Op::rloadu
            | Op::call => self.count_inst(),
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
            Op::mkclsr => self.fmt_clsr(),
        }
    }
}

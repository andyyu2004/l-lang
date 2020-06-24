use super::Function;
use crate::error::{VMError, VMResult};
use crate::{exec::Op, Type};
use std::convert::{TryFrom, TryInto};

/// stack frame
#[derive(Debug)]
pub struct Frame {
    /// instruction pointer
    pub(crate) ip: usize,
    /// stack pointer (index of top of the stack relative to the current frame)
    pub(crate) sp: usize,
    pub(crate) f: Function,
}

impl Frame {
    pub fn new(f: Function) -> Self {
        Self { f, ip: 0, sp: 0 }
    }

    pub fn read(&mut self, size: usize) -> &[u8] {
        self.ip += size;
        &self.f.code[self.ip - size..self.ip]
    }

    pub fn read_byte(&mut self) -> u8 {
        self.ip += 1;
        self.f.code[self.ip - 1]
    }

    pub fn read_opcode(&mut self) -> VMResult<Op> {
        Op::try_from(self.read_byte()).map_err(|e| VMError::InvalidOpcode(e.number))
    }

    pub fn read_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.read(8).try_into().unwrap())
    }

    pub fn read_type(&mut self) -> VMResult<Type> {
        Type::try_from(self.read_byte()).map_err(|e| VMError::InvalidType(e.number))
    }
}

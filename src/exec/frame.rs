use super::Function;
use crate::error::{VMError, VMResult};
use crate::{
    exec::Op, gc::{GCStateMap, Gc, Trace}, Type
};
use std::convert::{TryFrom, TryInto};

/// stack frame
#[derive(Debug)]
pub struct Frame {
    /// instruction pointer
    pub(crate) ip: usize,
    /// stack pointer (index of top of the stack relative to the current frame)
    pub(crate) sp: usize,
    /// the function the stack frame is executing
    pub(crate) f: Gc<Function>,
    /// return address as an index,
    pub(crate) ret_addr: usize,
}

impl Trace for Frame {
    fn mark(&self, map: &mut GCStateMap) {
        self.f.mark(map)
    }
}

impl Frame {
    pub fn new(f: Gc<Function>, ret_addr: usize) -> Self {
        Self {
            f,
            ip: 0,
            sp: 0,
            ret_addr,
        }
    }

    pub fn read(&mut self, size: usize) -> &[u8] {
        self.ip += size;
        &self.f.code[self.ip - size..self.ip]
    }

    pub fn read_byte(&mut self) -> u8 {
        let byte = self.f.code[self.ip];
        self.ip += 1;
        byte
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

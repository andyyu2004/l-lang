use super::Op;
use crate::util::As8Bytes;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Code(Vec<u8>);

#[derive(Default)]
pub struct CodeBuilder {
    code: Vec<u8>,
}

impl CodeBuilder {
    pub fn add_op(mut self, op: Op) -> Self {
        self.code.push(op as u8);
        self
    }

    pub fn add_const(mut self, c: impl As8Bytes) -> Self {
        self.code.extend_from_slice(&c.as_bytes());
        self
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

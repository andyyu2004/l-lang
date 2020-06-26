use super::Code;
use crate::gc::Trace;

#[derive(Debug)]
pub struct Function {
    pub(crate) code: Code,
    pub(crate) upvalc: u8,
}

impl Function {
    pub fn new(code: Code) -> Self {
        Self { code, upvalc: 0 }
    }
}

impl Trace for Function {
}

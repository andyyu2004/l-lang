use super::Code;
use crate::gc::Trace;

#[derive(Debug, Clone)]
pub struct Function {
    pub(crate) code: Code,
    pub(crate) upvalc: u8,
}

impl Function {
    pub fn new(code: Code) -> Self {
        Self::with_upvalc(code, 0)
    }

    pub fn with_upvalc(code: Code, upvalc: u8) -> Self {
        Self { code, upvalc }
    }
}

impl Trace for Function {
}

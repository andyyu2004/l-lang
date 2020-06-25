use super::Code;
use crate::gc::Trace;

#[derive(Debug)]
pub struct Function {
    pub(crate) code: Code,
}

impl Function {
    pub fn new(code: Code) -> Self {
        Self { code }
    }
}

impl Trace for Function {
}

use super::Code;
use crate::gc::Trace;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Function {
    pub(crate) code: Code,
}

impl Function {
    pub fn new(code: Code) -> Self {
        Self { code }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl Trace for Function {
}

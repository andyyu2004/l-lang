use super::Code;

#[derive(Debug)]
pub struct Function {
    pub(crate) code: Code,
}

impl Function {
    pub fn new(code: Code) -> Self {
        Self { code }
    }
}

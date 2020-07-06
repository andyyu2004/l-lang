use crate::{exec::Function, gc::Trace, impl_from_inner};

pub type ConstantPool = Vec<Constant>;

impl_from_inner!(Function, Constant, Function);
impl_from_inner!(String, Constant, String);

#[derive(Debug)]
pub enum Constant {
    Function(Function),
    String(String),
}

impl Trace for Constant {
}

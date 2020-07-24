use crate::gc::GCStateMap;
use crate::{exec::Function, gc::Trace, impl_from_inner};
use indexed_vec::IndexVec;

newtype_index!(ConstId);

pub type ConstantPool = IndexVec<ConstId, Constant>;

impl_from_inner!(Function, Constant, Function);
impl_from_inner!(String, Constant, String);

/// compiled constant
#[derive(Debug, Clone)]
pub enum Constant {
    Function(Function),
    String(String),
}

impl Constant {
    pub fn as_fn(self) -> Function {
        match self {
            Self::Function(f) => f,
            _ => panic!("expected fn constant found {:?}", self),
        }
    }
}

impl Trace for Constant {
    fn mark(&self, map: &mut GCStateMap) {
        match self {
            Self::Function(f) => f.mark(map),
            Self::String(_) => {}
        }
    }
}

use crate::gc::GCStateMap;
use crate::{exec::Function, gc::Trace, impl_from_inner};
use indexed_vec::IndexVec;
use std::fmt::{self, Display, Formatter};

newtype_index!(ConstId);
pub type ConstantPool = IndexVec<ConstId, Constant>;

impl_from_inner!(String, Constant, String);

/// compiled constant
#[derive(Debug, Clone)]
pub enum Constant {
    /// `Function` cannot capture upvars
    Function(Function),
    /// `Lambda` are closures and may capture upvars
    Lambda(Function),
    String(String),
}

impl Constant {
    pub fn as_fn(self) -> Function {
        match self {
            Self::Function(f) | Self::Lambda(f) => f,
            _ => panic!("expected fn constant found {:?}", self),
        }
    }
}

impl Display for Constant {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Lambda(f) => write!(fmt, "{}", f),
            Constant::Function(f) => write!(fmt, "{}", f),
            Constant::String(s) => write!(fmt, "{}", s),
        }
    }
}

impl Trace for Constant {
    fn mark(&self, map: &mut GCStateMap) {
        match self {
            Self::Lambda(f) => f.mark(map),
            Self::Function(f) => f.mark(map),
            Self::String(_) => {}
        }
    }
}

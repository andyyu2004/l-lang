#![feature(const_panic)]

mod def;
mod ty;

use std::fmt::{self, Debug, Display, Formatter};

use index::Idx;
pub use ty::*;

index::newtype_index!(
    pub struct DefId {
        DEBUG_FORMAT = "DefId({})"
    }
);

index::newtype_index!(
    pub struct LocalId {
        DEBUG_FORMAT = "LocalId({})"
    }
);

index::newtype_index!(
    pub struct ModuleId {
        DEBUG_FORMAT = "ModuleId({})",
        const ROOT_MODULE = 0
    }
);

index::newtype_index!(
    pub struct ParamIdx {
        DEBUG_FORMAT ="ParamIdx({})"
    }
);

index::newtype_index!(
    pub struct VariantIdx {
        DEBUG_FORMAT = "VariantIdx({})"
    }
);

index::newtype_index!(
    pub struct FieldIdx {
        DEBUG_FORMAT = "FieldIdx({})"
    }
);

impl DefId {
    pub fn dummy() -> Self {
        DefId::new(usize::MAX)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ImplItemId(DefId);

impl Display for LocalId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for DefId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    /// id of the immediately enclosing item
    pub def: DefId,
    /// id of node relative to the enclosing def_id
    pub local: LocalId,
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}.{:?}", self.def, self.local)
    }
}

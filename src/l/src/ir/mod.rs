mod ast_lowering;
mod def;
mod expr;
mod ir;
mod item;
mod item_visitor;
mod map;
mod pattern;
mod prog;
mod stmt;
mod ty;
mod visit;

pub use ast_lowering::AstLoweringCtx;
pub use def::*;
pub use expr::{Expr, ExprKind};
use indexed_vec::Idx;
pub use ir::*;
pub use item::*;
pub use item_visitor::*;
pub use pattern::{Pattern, PatternKind};
pub use prog::IR;
use std::fmt::{self, Display, Formatter};
pub use stmt::{Stmt, StmtKind};
pub use ty::*;
pub use visit::*;

newtype_index!(DefId);
newtype_index!(LocalId);
newtype_index!(ModuleId);
newtype_index!(ParamIdx);
newtype_index!(VariantIdx);
newtype_index!(FieldIdx);

impl DefId {
    pub fn dummy() -> Self {
        DefId::new(usize::MAX)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ImplItemId(DefId);

pub const ROOT_MODULE: ModuleId = ModuleId(0);

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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Id {
    /// id of the immediately enclosing item
    pub def: DefId,
    /// id of node relative to the enclosing def_id
    pub local: LocalId,
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}.{:?}", self.def, self.local)
    }
}
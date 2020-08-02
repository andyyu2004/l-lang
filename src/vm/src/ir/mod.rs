mod ast_lowering;
mod def;
mod expr;
mod ir;
mod item;
mod pattern;
mod prog;
mod stmt;
mod ty;
mod visit;

crate use ast_lowering::AstLoweringCtx;
crate use def::*;
crate use expr::{Expr, ExprKind};
use indexed_vec::Idx;
crate use ir::*;
crate use item::{Item, ItemKind};
crate use pattern::{Pattern, PatternKind};
crate use prog::Prog;
use std::fmt::{self, Display, Formatter};
crate use stmt::{Stmt, StmtKind};
crate use ty::*;
crate use visit::*;

newtype_index!(DefId);
newtype_index!(LocalId);

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
crate struct Id {
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

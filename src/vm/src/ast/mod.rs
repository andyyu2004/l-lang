mod ast;
mod expr;
mod item;
mod pattern;
mod prog;
mod stmt;
mod ty;
mod visit;

newtype_index!(NodeId);

pub type P<T> = Box<T>;

pub use ast::*;
pub use expr::{Expr, ExprKind};
pub use item::{Item, ItemKind};
pub use pattern::{Pattern, PatternKind};
pub use prog::Prog;
pub use stmt::{Let, Stmt, StmtKind};
pub use ty::{Ty, TyKind};
pub use visit::*;

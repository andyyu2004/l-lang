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
pub use expr::*;
pub use item::*;
pub use pattern::*;
pub use prog::Prog;
pub use stmt::*;
pub use ty::*;
pub use visit::*;

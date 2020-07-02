mod ast;
mod expr;
mod item;
mod prog;
mod ty;

crate type P<T> = Box<T>;

crate use ast::*;
crate use expr::{Expr, ExprKind};
crate use item::{Item, ItemKind};
crate use prog::Prog;
crate use ty::{Ty, TyKind};

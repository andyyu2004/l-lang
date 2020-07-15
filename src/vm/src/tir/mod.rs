//! typed ir
mod expr;
mod ir_lowering;
mod item;
mod pattern;
mod prog;
mod stmt;
mod tir;
mod visitor;

crate use expr::{Expr, ExprKind};
crate use ir_lowering::IrLoweringCtx;
crate use item::{Item, ItemKind};
crate use pattern::{Field, FieldPat, Pattern, PatternKind};
crate use prog::Prog;
crate use stmt::{Stmt, StmtKind};
crate use tir::*;
crate use visitor::Visitor;

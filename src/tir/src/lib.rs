//! typed ir

#![feature(split_inclusive)]

#[macro_use]
extern crate log;

mod expr;
mod fmt;
mod ir_lowering;
mod item;
mod map;
mod pattern;
mod prog;
mod stmt;
mod visitor;

pub use expr::{Expr, ExprKind};
pub use fmt::Formatter;
pub use ir_lowering::TirCtx;
pub use item::{Item, ItemKind};
pub use map::Map;
pub use pattern::{FieldPat, Pattern, PatternKind};
pub use prog::Prog;
pub use stmt::{Stmt, StmtKind};
pub use visitor::Visitor;

use crate as tir;
use ast::Ident;
use ir::{self, FieldIdx, Id, Res};
use lcore::ty::{List, SubstsRef, Ty};
use span::Span;
use std::marker::PhantomData;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct BodyId(ir::Id);

#[derive(Debug)]
pub struct Generics<'tcx> {
    /// just to make it not a ZST
    pub data: usize,
    pub pd: PhantomData<&'tcx ()>,
}

#[derive(Debug)]
pub struct Field<'tcx> {
    pub index: FieldIdx,
    pub ident: Ident,
    pub expr: &'tcx tir::Expr<'tcx>,
}

#[derive(Debug)]
pub struct Arm<'tcx> {
    pub id: ir::Id,
    pub span: Span,
    pub pat: &'tcx tir::Pattern<'tcx>,
    pub guard: Option<&'tcx tir::Expr<'tcx>>,
    pub body: &'tcx tir::Expr<'tcx>,
}

impl<'tcx> std::fmt::Display for Arm<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} => {}", self.pat, self.body)
    }
}

#[derive(Debug)]
pub struct Body<'tcx> {
    pub params: &'tcx [tir::Param<'tcx>],
    pub expr: &'tcx tir::Expr<'tcx>,
}

impl<'tcx> std::fmt::Display for Body<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)
    }
}

#[derive(Debug)]
pub struct Path<'tcx> {
    pub span: Span,
    pub res: Res,
    pub segments: &'tcx [PathSegment<'tcx>],
}

#[derive(Debug)]
pub struct Param<'tcx> {
    pub id: ir::Id,
    pub span: Span,
    pub pat: &'tcx tir::Pattern<'tcx>,
}

impl<'tcx> std::fmt::Display for Param<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pat)
    }
}

#[derive(Debug)]
pub struct PathSegment<'tcx> {
    pub ident: Ident,
    pd: PhantomData<&'tcx ()>,
}

#[derive(Debug)]
pub struct Block<'tcx> {
    pub id: Id,
    pub stmts: &'tcx [tir::Stmt<'tcx>],
    pub expr: Option<&'tcx tir::Expr<'tcx>>,
}

impl<'tcx> std::fmt::Display for Block<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for stmt in self.stmts {
            writeln!(f, "\t{};", stmt)?;
        }
        self.expr.map(|expr| writeln!(f, "\t{}", expr));
        write!(f, "}}")
    }
}

#[derive(Debug)]
pub struct Let<'tcx> {
    pub id: Id,
    pub pat: &'tcx tir::Pattern<'tcx>,
    pub init: Option<&'tcx tir::Expr<'tcx>>,
}

impl<'tcx> std::fmt::Display for Let<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {}", self.pat)?;
        self.init.map(|init| write!(f, " = {}", init));
        Ok(())
    }
}

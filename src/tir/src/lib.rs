//! typed ir

#![feature(box_syntax, box_patterns)]

mod expr;
mod fmt;
mod item;
mod pattern;
mod prog;
mod stmt;
mod visitor;

pub use expr::{Expr, ExprKind};
pub use fmt::Formatter;
pub use item::{Item, ItemKind};
pub use pattern::{FieldPat, Pattern, PatternKind};
pub use prog::Prog;
pub use stmt::{Stmt, StmtKind};
pub use visitor::Visitor;

use crate as tir;
use ast::Ident;
use ir::{self, FieldIdx, Id, Res};
use lcore::ty::Ty;
use span::Span;
use std::marker::PhantomData;

impl<'tcx> std::fmt::Display for Arm<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} => {}", self.pat, self.body)
    }
}

impl<'tcx> std::fmt::Display for Body<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)
    }
}

impl<'tcx> std::fmt::Display for Param<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pat)
    }
}

impl<'tcx> std::fmt::Display for Block<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for stmt in &self.stmts {
            writeln!(f, "\t{};", stmt)?;
        }
        self.expr.as_ref().map(|expr| writeln!(f, "\t{}", expr));
        write!(f, "}}")
    }
}

impl<'tcx> std::fmt::Display for Let<'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {}", self.pat)?;
        self.init.as_ref().map(|init| write!(f, " = {}", init));
        Ok(())
    }
}

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
    pub expr: Box<tir::Expr<'tcx>>,
}

#[derive(Debug)]
pub struct Arm<'tcx> {
    pub id: ir::Id,
    pub span: Span,
    pub pat: Box<tir::Pattern<'tcx>>,
    pub guard: Option<Box<tir::Expr<'tcx>>>,
    pub body: Box<tir::Expr<'tcx>>,
}

#[derive(Debug)]
pub struct Body<'tcx> {
    pub params: Vec<tir::Param<'tcx>>,
    pub expr: Box<tir::Expr<'tcx>>,
}

#[derive(Debug)]
pub struct Path<'tcx> {
    pub span: Span,
    pub res: Res,
    pub segments: Vec<PathSegment<'tcx>>,
}

#[derive(Debug)]
pub struct Param<'tcx> {
    pub id: ir::Id,
    pub span: Span,
    pub pat: Box<tir::Pattern<'tcx>>,
}

#[derive(Debug)]
pub struct PathSegment<'tcx> {
    pub ident: Ident,
    pd: PhantomData<&'tcx ()>,
}

#[derive(Debug)]
pub struct Block<'tcx> {
    pub id: Id,
    pub stmts: Vec<tir::Stmt<'tcx>>,
    pub expr: Option<Box<tir::Expr<'tcx>>>,
}

#[derive(Debug)]
pub struct Let<'tcx> {
    pub id: Id,
    pub pat: Box<tir::Pattern<'tcx>>,
    pub init: Option<Box<tir::Expr<'tcx>>>,
}

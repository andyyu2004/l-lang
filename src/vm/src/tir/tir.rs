use crate::ir;
use crate::tir;
use crate::ty::{List, SubstRef, Ty};
use crate::{ast::Ident, span::Span};
use ir::{Id, Res};
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
crate struct BodyId(ir::Id);

#[derive(Debug)]
crate struct Generics<'tcx> {
    /// just to make it not a ZST
    pub data: usize,
    pub pd: PhantomData<&'tcx ()>,
}

#[derive(Debug)]
crate struct Arm<'tcx> {
    pub id: ir::Id,
    pub span: Span,
    pub pat: &'tcx tir::Pattern<'tcx>,
    pub guard: Option<&'tcx tir::Expr<'tcx>>,
    pub body: &'tcx tir::Expr<'tcx>,
}

impl<'tcx> Display for Arm<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}", self.pat, self.body)
    }
}

#[derive(Debug)]
crate struct Body<'tcx> {
    pub params: &'tcx [tir::Param<'tcx>],
    pub expr: &'tcx tir::Expr<'tcx>,
}

impl<'tcx> Display for Body<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expr)
    }
}

#[derive(Debug)]
crate struct Path<'tcx> {
    pub span: Span,
    pub res: Res,
    pub segments: &'tcx [PathSegment<'tcx>],
}

#[derive(Debug)]
crate struct Param<'tcx> {
    pub id: ir::Id,
    pub span: Span,
    pub pat: &'tcx tir::Pattern<'tcx>,
}

impl<'tcx> Display for Param<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pat)
    }
}

#[derive(Debug)]
crate struct PathSegment<'tcx> {
    pub ident: Ident,
    pd: PhantomData<&'tcx ()>,
}

#[derive(Debug)]
crate struct Block<'tcx> {
    pub id: Id,
    pub stmts: &'tcx [tir::Stmt<'tcx>],
    pub expr: Option<&'tcx tir::Expr<'tcx>>,
}

impl<'tcx> Display for Block<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        self.stmts.iter().map(|stmt| writeln!(f, "\t{};", stmt)).count();
        self.expr.map(|expr| writeln!(f, "\t{}", expr));
        write!(f, "}}")
    }
}

#[derive(Debug)]
crate struct Let<'tcx> {
    pub id: Id,
    pub pat: &'tcx tir::Pattern<'tcx>,
    pub init: Option<&'tcx tir::Expr<'tcx>>,
}

impl<'tcx> Display for Let<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "let {}", self.pat)?;
        self.init.map(|init| write!(f, " = {}", init));
        Ok(())
    }
}

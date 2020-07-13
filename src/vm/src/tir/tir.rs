use crate::ir;
use crate::tir;
use crate::ty::Ty;
use crate::{ast::Ident, span::Span, typeck::List};
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
        self.stmts.iter().map(|stmt| writeln!(f, "{}", stmt)).count();
        self.expr.as_ref().map(|expr| writeln!(f, "{}", expr));
        Ok(())
    }
}

#[derive(Debug)]
crate struct Let<'tcx> {
    pub id: Id,
    pub pat: &'tcx tir::Pattern<'tcx>,
    pub ty: Option<&'tcx Ty<'tcx>>,
    pub init: Option<&'tcx tir::Expr<'tcx>>,
}

impl<'tcx> Display for Let<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pat)?;
        self.ty.as_ref().map(|ty| write!(f, ": {}", ty));
        self.init.as_ref().map(|init| write!(f, " = {}", init));
        Ok(())
    }
}

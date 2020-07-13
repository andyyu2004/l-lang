use crate::ir::Id;
use crate::span::Span;
use crate::tir;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
crate struct Stmt<'tcx> {
    pub id: Id,
    pub span: Span,
    pub kind: tir::StmtKind<'tcx>,
}

impl<'tcx> Display for Stmt<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug)]
crate enum StmtKind<'tcx> {
    Let(&'tcx tir::Let<'tcx>),
    Expr(&'tcx tir::Expr<'tcx>),
}

impl<'tcx> Display for StmtKind<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StmtKind::Let(l) => write!(f, "{}", l),
            StmtKind::Expr(expr) => write!(f, "{}", expr),
        }
    }
}

use crate::ir::Id;
use crate::span::Span;
use crate::tir;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Stmt<'tcx> {
    pub id: Id,
    pub span: Span,
    pub kind: tir::StmtKind<'tcx>,
}

impl<'tcx> Display for Stmt<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        tir::Formatter::new(f).fmt_stmt(self)
    }
}

#[derive(Debug)]
pub enum StmtKind<'tcx> {
    Let(&'tcx tir::Let<'tcx>),
    Expr(&'tcx tir::Expr<'tcx>),
}

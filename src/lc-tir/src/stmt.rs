use crate as tir;
use ir::Id;
use lc_span::Span;
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
    Let(tir::Let<'tcx>),
    Expr(Box<tir::Expr<'tcx>>),
}

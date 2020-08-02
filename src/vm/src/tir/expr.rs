use crate::ir::DefId;
use crate::span::Span;
use crate::ty::{Const, Ty};
use crate::{ast, ir, tir, util};
use fmt::Display;
use std::fmt::{self, Formatter};

#[derive(Debug)]
crate struct Expr<'tcx> {
    pub ty: Ty<'tcx>,
    pub span: Span,
    pub kind: tir::ExprKind<'tcx>,
}

#[derive(Debug)]
crate enum ExprKind<'tcx> {
    Const(&'tcx Const<'tcx>),
    Bin(ast::BinOp, &'tcx tir::Expr<'tcx>, &'tcx tir::Expr<'tcx>),
    Unary(ast::UnaryOp, &'tcx tir::Expr<'tcx>),
    Block(&'tcx tir::Block<'tcx>),
    /// reference to a local variable (reference not in the rust sense, but just a usage of the variable)
    VarRef(ir::Id),
    /// reference to an item such as a function or a constant
    ItemRef(DefId),
    Tuple(&'tcx [tir::Expr<'tcx>]),
    Lambda(&'tcx tir::Body<'tcx>),
    Call(&'tcx tir::Expr<'tcx>, &'tcx [tir::Expr<'tcx>]),
    Match(&'tcx tir::Expr<'tcx>, &'tcx [tir::Arm<'tcx>]),
}

impl<'tcx> Display for Expr<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        tir::Formatter::new(f).fmt_expr(self)
    }
}

use crate::ir;
use crate::tir;
use crate::util;
use crate::{ast, span::Span, ty::Ty};
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
    Lit(ast::Lit),
    Bin(ast::BinOp, &'tcx tir::Expr<'tcx>, &'tcx tir::Expr<'tcx>),
    Unary(ast::UnaryOp, &'tcx tir::Expr<'tcx>),
    Block(&'tcx tir::Block<'tcx>),
    /// reference to a local variable (reference not in the rust sense, but just a usage of the variable)
    VarRef(ir::Id),
    Tuple(&'tcx [tir::Expr<'tcx>]),
    Lambda(&'tcx tir::Body<'tcx>),
    Call(&'tcx tir::Expr<'tcx>, &'tcx [tir::Expr<'tcx>]),
}

impl<'tcx> Display for Expr<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind, self.ty)
    }
}

impl<'tcx> Display for ExprKind<'tcx> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lit(l) => write!(fmt, "{}", l),
            Self::Bin(op, l, r) => write!(fmt, "({} {} {})", op, l, r),
            Self::Unary(op, expr) => write!(fmt, "({} {})", op, expr),
            Self::Block(block) => write!(fmt, "{}", block),
            Self::VarRef(id) => write!(fmt, "${:?}", id.local),
            Self::Tuple(xs) => write!(fmt, "({})", util::join2(xs.iter(), ",")),
            Self::Lambda(b) => write!(fmt, "(λ({}) {})", util::join2(b.params.iter(), ","), b.expr),
            Self::Call(f, args) => write!(fmt, "({} {})", f, util::join2(args.iter(), " ")),
        }
    }
}

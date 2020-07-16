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
}

impl<'tcx> Display for Expr<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind, self.ty)
    }
}

impl<'tcx> Display for ExprKind<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lit(l) => write!(f, "{}", l),
            Self::Bin(op, l, r) => write!(f, "({} {} {})", op, l, r),
            Self::Unary(op, expr) => write!(f, "({} {})", op, expr),
            Self::Block(block) => write!(f, "{}", block),
            Self::VarRef(id) => write!(f, "${:?}", id.local),
            Self::Tuple(xs) => write!(f, "({})", util::join2(xs.iter(), ",")),
            Self::Lambda(b) => {
                write!(f, "(λ({}) => {})", util::join2(b.params.iter(), ","), b.expr)
            }
        }
    }
}

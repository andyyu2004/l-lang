use crate::{ast, lexer::Span, ty::Ty};
use fmt::Display;
use std::fmt::{self, Formatter};

#[derive(Debug)]
crate struct Expr<'tcx> {
    pub ty: Ty<'tcx>,
    pub span: Span,
    pub kind: ExprKind<'tcx>,
}

impl Default for Expr<'_> {
    fn default() -> Self {
        todo!()
    }
}

#[derive(Debug)]
crate enum ExprKind<'tcx> {
    Lit(ast::Lit),
    Bin(ast::BinOp, &'tcx Expr<'tcx>, &'tcx Expr<'tcx>),
    Unary(ast::UnaryOp, &'tcx Expr<'tcx>),
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
        }
    }
}

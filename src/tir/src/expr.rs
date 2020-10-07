use crate as tir;
use fmt::Display;
use ir::{DefId, FieldIdx, VariantIdx};
use lcore::ty::{AdtTy, Const, SubstsRef, Ty};
use span::Span;
use std::fmt::{self, Formatter};

#[derive(Debug)]
pub struct Expr<'tcx> {
    pub ty: Ty<'tcx>,
    pub span: Span,
    pub kind: tir::ExprKind<'tcx>,
}

#[derive(Debug)]
pub enum ExprKind<'tcx> {
    Const(&'tcx Const<'tcx>),
    Bin(ast::BinOp, &'tcx tir::Expr<'tcx>, &'tcx tir::Expr<'tcx>),
    Unary(ast::UnaryOp, &'tcx tir::Expr<'tcx>),
    Block(&'tcx tir::Block<'tcx>),
    /// reference to a local variable
    /// (reference not in the & sense, but just a usage of the variable)
    VarRef(ir::Id),
    /// reference to an item such as a function item or a constant
    ItemRef(DefId),
    /// (x, y)
    Tuple(&'tcx [tir::Expr<'tcx>]),
    /// f(x)
    Call(&'tcx tir::Expr<'tcx>, &'tcx [tir::Expr<'tcx>]),
    Match(&'tcx tir::Expr<'tcx>, &'tcx [tir::Arm<'tcx>]),
    /// x = y
    Assign(&'tcx tir::Expr<'tcx>, &'tcx tir::Expr<'tcx>),
    /// s.x
    Field(&'tcx tir::Expr<'tcx>, FieldIdx),
    /// return x
    Ret(Option<&'tcx tir::Expr<'tcx>>),
    /// &x
    Ref(&'tcx tir::Expr<'tcx>),
    /// *x
    Deref(&'tcx tir::Expr<'tcx>),
    /// box x
    Box(&'tcx tir::Expr<'tcx>),
    Closure {
        upvars: &'tcx [tir::Expr<'tcx>],
        body: &'tcx tir::Body<'tcx>,
    },
    Adt {
        adt: &'tcx AdtTy<'tcx>,
        variant_idx: VariantIdx,
        substs: SubstsRef<'tcx>,
        fields: &'tcx [tir::Field<'tcx>],
    },
}

impl<'tcx> Display for Expr<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        tir::Formatter::new(f).fmt_expr(self)
    }
}

use crate::ir::{DefId, FieldIdx, VariantIdx};
use crate::span::Span;
use crate::ty::{AdtTy, Const, SubstsRef, Ty};
use crate::{ast, ir, tir, util};
use ast::Ident;
use fmt::Display;
use std::fmt::{self, Formatter};

#[derive(Debug)]
pub struct Expr<'tcx> {
    pub id: ir::Id,
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
    /// reference to a local variable (reference not in the rust sense, but just a usage of the variable)
    VarRef(ir::Id),
    /// reference to an item such as a function or a constant
    ItemRef(DefId),
    Tuple(&'tcx [tir::Expr<'tcx>]),
    Lambda(&'tcx tir::Body<'tcx>),
    Call(&'tcx tir::Expr<'tcx>, &'tcx [tir::Expr<'tcx>]),
    Match(&'tcx tir::Expr<'tcx>, &'tcx [tir::Arm<'tcx>]),
    Assign(&'tcx tir::Expr<'tcx>, &'tcx tir::Expr<'tcx>),
    Field(&'tcx tir::Expr<'tcx>, FieldIdx),
    Ret(Option<&'tcx tir::Expr<'tcx>>),
    Ref(&'tcx tir::Expr<'tcx>),
    Deref(&'tcx tir::Expr<'tcx>),
    Box(&'tcx tir::Expr<'tcx>),
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

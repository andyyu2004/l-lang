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
    Bin(ast::BinOp, Box<tir::Expr<'tcx>>, Box<tir::Expr<'tcx>>),
    Unary(ast::UnaryOp, Box<tir::Expr<'tcx>>),
    Block(Box<tir::Block<'tcx>>),
    /// reference to a local variable
    /// (reference not in the & sense, but just a usage of the variable)
    VarRef(ir::Id),
    /// reference to an item such as a function item or a constant
    /// the second field is the substituions used to obtain the "concrete"
    /// type of the item
    /// "concrete" because there may still be type parameters in these substs
    ItemRef(DefId, SubstsRef<'tcx>),
    /// (x, y)
    Tuple(Vec<tir::Expr<'tcx>>),
    /// f(x)
    Call(Box<tir::Expr<'tcx>>, Vec<tir::Expr<'tcx>>),
    Match(Box<tir::Expr<'tcx>>, Vec<tir::Arm<'tcx>>),
    /// x = y
    Assign(Box<tir::Expr<'tcx>>, Box<tir::Expr<'tcx>>),
    /// s.x
    Field(Box<tir::Expr<'tcx>>, FieldIdx),
    /// return x
    Ret(Option<Box<tir::Expr<'tcx>>>),
    /// &x
    Ref(Box<tir::Expr<'tcx>>),
    /// *x
    Deref(Box<tir::Expr<'tcx>>),
    /// box x
    Box(Box<tir::Expr<'tcx>>),
    Closure {
        body: Box<tir::Body<'tcx>>,
        upvars: Vec<tir::Expr<'tcx>>,
    },
    Adt {
        adt: &'tcx AdtTy,
        variant_idx: VariantIdx,
        substs: SubstsRef<'tcx>,
        fields: Vec<tir::Field<'tcx>>,
    },
}

impl<'tcx> Display for Expr<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        tir::Formatter::new(f).fmt_expr(self)
    }
}

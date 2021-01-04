use crate as tir;
use ast::{Ident, Mutability};
use ir::{self, FieldIdx, VariantIdx};
use lcore::ty::{AdtTy, Const, SubstsRef, Ty};
use span::Span;
use std::fmt::{self, Display, Formatter};
use util;

#[derive(Debug)]
pub struct FieldPat<'tcx> {
    pub index: FieldIdx,
    pub pat: Box<Pattern<'tcx>>,
}

impl<'tcx> Display for FieldPat<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pat)
    }
}

#[derive(Debug)]
pub struct Pattern<'tcx> {
    pub id: ir::Id,
    pub span: Span,
    pub ty: Ty<'tcx>,
    pub kind: tir::PatternKind<'tcx>,
}

impl<'tcx> Pattern<'tcx> {
    pub fn is_irrefutable(&self) -> bool {
        !self.is_refutable()
    }

    pub fn is_refutable(&self) -> bool {
        match &self.kind {
            PatternKind::Box(pat) => pat.is_refutable(),
            PatternKind::Field(fs) => fs.iter().any(|f| f.pat.is_refutable()),
            PatternKind::Lit(..) => true,
            PatternKind::Variant(.., pats) => pats.iter().any(|p| p.is_refutable()),
            PatternKind::Wildcard | PatternKind::Binding(..) => false,
        }
    }
}

impl<'tcx> Display for Pattern<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            // we print out the `local_id` instead of the ident symbol number
            // as the identifier is referred to by id instead of name in the tir
            // in particular ExprKind::VarRef does not have access to the symbol only the `ir::Id`
            PatternKind::Box(pat) => write!(f, "&{}", pat),
            PatternKind::Binding(m, ident, _) => write!(f, "{}{}", m, ident),
            PatternKind::Field(fields) => write!(f, "({})", lutil::join2(fields.iter(), ",")),
            PatternKind::Lit(expr) => {
                // don't double print the type as expr will already do so
                assert_eq!(expr.ty, self.ty);
                return write!(f, "{}", expr);
            }
            PatternKind::Wildcard => write!(f, "_"),
            PatternKind::Variant(adt_ty, substs, variant_idx, pats) => write!(
                f,
                "{}::{}<{}>({})",
                adt_ty.ident,
                adt_ty.variants[*variant_idx].ident,
                substs,
                lutil::join2(pats.iter(), ","),
            ),
        }?;
        write!(f, ":{}", self.ty)
    }
}

#[derive(Debug)]
pub enum PatternKind<'tcx> {
    /// &<pat>
    Box(Box<tir::Pattern<'tcx>>),
    Binding(Mutability, Ident, Option<Box<tir::Pattern<'tcx>>>),
    /// generalization of tuple patterns
    /// used to match tuples, and single variant adts (i.e. structs)
    /// `(...)`, `Foo(...)`, `Foo{...}`, or `Foo`, where `Foo` is a variant name from an ADT with
    /// a single variant.
    Field(Vec<tir::FieldPat<'tcx>>),
    Lit(&'tcx Const<'tcx>),
    /// `Foo(...)` or `Foo {...}` or `Foo`, where `Foo` is a variant name from an ADT with multiple variants.
    Variant(&'tcx AdtTy, SubstsRef<'tcx>, VariantIdx, Vec<tir::Pattern<'tcx>>),
    Wildcard,
}

use crate::ir;
use crate::span::Span;
use crate::ty::{Const, Ty};
use crate::util;
use crate::{ast::Ident, tir};
use std::fmt::{self, Display, Formatter};

newtype_index!(Field);

#[derive(Debug)]
pub struct FieldPat<'tcx> {
    pub field: Field,
    pub pat: &'tcx Pattern<'tcx>,
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

impl<'tcx> Display for Pattern<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            // we print out the `local_id` instead of the ident symbol number
            // as the identifier is referred to by id instead of name in the tir
            // in particular ExprKind::VarRef does not have access to the symbol only the `ir::Id`
            PatternKind::Binding(ident, _) => write!(f, "{}", ident),
            PatternKind::Field(fields) => write!(f, "({})", util::join2(fields.iter(), ",")),
            PatternKind::Lit(expr) => {
                // don't double print the type as expr will already do so
                assert_eq!(expr.ty, self.ty);
                return write!(f, "{}", expr);
            }
            PatternKind::Wildcard => write!(f, "_"),
        }?;
        write!(f, ":{}", self.ty)
    }
}

#[derive(Debug)]
pub enum PatternKind<'tcx> {
    Wildcard,
    Binding(Ident, Option<&'tcx tir::Pattern<'tcx>>),
    /// generalization of tuple patterns
    Field(&'tcx [tir::FieldPat<'tcx>]),
    Lit(&'tcx tir::Expr<'tcx>),
}

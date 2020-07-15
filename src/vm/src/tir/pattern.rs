use crate::ir;
use crate::span::Span;
use crate::{ast::Ident, tir, ty::Ty};
use std::fmt::{self, Display, Formatter};

newtype_index!(Field);

#[derive(Debug)]
crate struct FieldPat<'tcx> {
    pub field: Field,
    pub pat: Pattern<'tcx>,
}

#[derive(Debug)]
crate struct Pattern<'tcx> {
    pub id: ir::Id,
    pub span: Span,
    pub ty: Ty<'tcx>,
    pub kind: tir::PatternKind<'tcx>,
}

impl<'tcx> Display for Pattern<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            PatternKind::Wildcard => write!(f, "_"),
            // we print out the `local_id` instead of the ident symbol number
            // as the identifier is referred to by id not name in the tir
            PatternKind::Binding(_, _) => write!(f, "${:?}", self.id.local),
            PatternKind::Field(_) => todo!(),
        }?;
        write!(f, ": {}", self.ty)
    }
}

#[derive(Debug)]
crate enum PatternKind<'tcx> {
    Wildcard,
    Binding(Ident, Option<&'tcx tir::Pattern<'tcx>>),
    /// generalization of tuple patterns
    Field(&'tcx [tir::FieldPat<'tcx>]),
}

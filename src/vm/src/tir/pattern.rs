use crate::ir::Id;
use crate::span::Span;
use crate::{ast::Ident, tir, ty::Ty};
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
crate struct Pattern<'tcx> {
    pub id: Id,
    pub span: Span,
    pub ty: Ty<'tcx>,
    pub kind: tir::PatternKind<'tcx>,
}

impl<'tcx> Display for Pattern<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.ty)
    }
}

#[derive(Debug)]
crate enum PatternKind<'tcx> {
    Wildcard,
    Binding(Ident, Option<&'tcx tir::Pattern<'tcx>>),
}

impl<'tcx> Display for PatternKind<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PatternKind::Wildcard => write!(f, "_"),
            PatternKind::Binding(ident, _sub) => write!(f, "{}", ident),
        }
    }
}

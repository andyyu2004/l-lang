use super::Subst;
use crate::{lexer::Span, ty::Ty, typeck::InferResult, util};
use std::{
    fmt::{self, Display, Formatter}, ops::{Deref, DerefMut}
};

#[derive(Default)]
crate struct Constraints<'tcx>(Vec<Constraint<'tcx>>);

impl<'tcx> Deref for Constraints<'tcx> {
    type Target = Vec<Constraint<'tcx>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'tcx> DerefMut for Constraints<'tcx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'tcx> Display for Constraints<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", util::join(&self.0, " & "))
    }
}

#[derive(Debug)]
crate struct Constraint<'tcx> {
    pub span: Span,
    pub kind: ConstraintKind<'tcx>,
}

impl<'tcx> Display for Constraint<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl<'tcx> Constraint<'tcx> {
    pub fn eq(span: Span, x: Ty<'tcx>, y: Ty<'tcx>) -> Self {
        Self {
            span,
            kind: ConstraintKind::Eq(x, y),
        }
    }
}

#[derive(Debug)]
crate enum ConstraintKind<'tcx> {
    Eq(Ty<'tcx>, Ty<'tcx>),
}

impl<'tcx> Display for ConstraintKind<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eq(s, t) => write!(f, "{} ~ {}", s, t),
        }
    }
}

use super::{Constraint, InferCtx};
use crate::{lexer::Span, ty::Ty};

crate struct At<'a, 'tcx> {
    pub span: Span,
    pub infcx: &'a InferCtx<'a, 'tcx>,
}

impl<'a, 'tcx> InferCtx<'a, 'tcx> {
    pub fn at(&'a self, span: Span) -> At<'a, 'tcx> {
        At { infcx: self, span }
    }
}

impl<'a, 'tcx> At<'a, 'tcx> {
    /// adds the constraint that `x` and `y` are to be equal
    pub fn ceq(&self, ty: Ty<'tcx>, expected: Ty<'tcx>) {
        self.infcx
            .constrain(Constraint::eq(self.span, ty, expected));
    }
}

use super::InferCtx;
use crate::span::Span;
use crate::ty::{self, Ty};
use crate::typeck::{InferError, InferResult};

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
    pub fn ceq(&self, a: Ty<'tcx>, b: Ty<'tcx>) -> InferResult<'tcx, Ty<'tcx>> {
        // todo keep the span somewhere
        let mut inner = self.infcx.inner.borrow_mut();
        let mut type_vars = inner.type_variables();

        match (&a.kind, &b.kind) {
            (&ty::Infer(ty::TyVar(a_id)), &ty::Infer(ty::TyVar(b_id))) => {
                type_vars.equate(a_id, b_id)
            }
            (&ty::Infer(ty::TyVar(vid)), _) => type_vars.instantiate(vid, b),
            (_, &ty::Infer(ty::TyVar(vid))) => type_vars.instantiate(vid, a),
            _ if a == b => Ok(()),
            _ => Err(InferError::UnificationFailure(a, b)),
        }?;

        Ok(a)
    }
}

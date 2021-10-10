use super::{Equate, InferCtx};
use lc_core::ty::*;
use lc_span::Span;
use std::ops::Deref;

/// perform an operation "at" some particular span
pub struct At<'a, 'tcx> {
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
    pub fn equate<T>(&self, a: T, b: T) -> TypeResult<'tcx, T>
    where
        T: Relate<'tcx>,
    {
        Equate { at: self }.relate(a, b)
    }
}

impl<'a, 'tcx> Deref for At<'a, 'tcx> {
    type Target = InferCtx<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.infcx
    }
}

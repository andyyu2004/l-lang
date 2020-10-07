use super::{Equate, InferCtx};
use lcore::ty::*;
use span::Span;

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

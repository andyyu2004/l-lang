use super::{FnCtx, InferCtx};
use crate::span::Span;
use crate::ty::{Adjuster, Adjustment, Ty, TyKind};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn autoderef(&'a self, span: Span, base: Ty<'tcx>) -> Autoderef<'a, 'tcx> {
        Autoderef::new(self, span, base)
    }
}

pub struct Autoderef<'a, 'tcx> {
    infcx: &'a InferCtx<'a, 'tcx>,
    span: Span,
    base_ty: Ty<'tcx>,
    curr_ty: Option<Ty<'tcx>>,
    adjustments: Vec<Adjustment<'tcx>>,
}

impl<'a, 'tcx> Autoderef<'a, 'tcx> {
    pub fn new(infcx: &'a InferCtx<'a, 'tcx>, span: Span, base_ty: Ty<'tcx>) -> Self {
        // TODO may need to partially resolve `base_ty` at this point?
        Self { infcx, span, base_ty, curr_ty: Some(base_ty), adjustments: Default::default() }
    }
}

impl<'tcx> Adjuster<'tcx> for Autoderef<'_, 'tcx> {
    fn get_adjustments(&self) -> Vec<Adjustment<'tcx>> {
        self.adjustments.to_vec()
    }
}

impl<'a, 'tcx> Iterator for &mut Autoderef<'a, 'tcx> {
    type Item = Ty<'tcx>;

    fn next(&mut self) -> Option<Self::Item> {
        let ty = self.curr_ty?;
        self.curr_ty = match ty.kind {
            TyKind::Ptr(_, ty) => {
                self.adjustments.push(Adjustment::new_deref(ty));
                Some(ty)
            }
            _ => None,
        };
        Some(ty)
    }
}
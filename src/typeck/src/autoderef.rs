use crate::FnCtx;
use infer::InferCtx;
use lcore::ty::{Adjuster, Adjustment, Ty, TyKind};
use span::Span;

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
        let base_ty = infcx.partially_resolve_ty(span, base_ty);
        Self { infcx, span, base_ty, curr_ty: Some(base_ty), adjustments: Default::default() }
    }
}

impl<'tcx> Adjuster<'tcx> for Autoderef<'_, 'tcx> {
    fn get_adjustments(&self) -> Vec<Adjustment<'tcx>> {
        self.adjustments.to_vec()
    }
}

impl<'a, 'tcx> Iterator for Autoderef<'a, 'tcx> {
    type Item = Ty<'tcx>;

    fn next(&mut self) -> Option<Self::Item> {
        let ty = self.curr_ty?;
        self.curr_ty = match ty.kind {
            // do not autoderef on raw pointer
            TyKind::Box(ty) => {
                self.adjustments.push(Adjustment::new_deref(ty));
                Some(ty)
            }
            _ => None,
        };
        Some(ty)
    }
}

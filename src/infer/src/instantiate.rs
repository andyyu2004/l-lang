use crate::InferCtx;
use lcore::ty::*;
use lcore::TyCtx;
use span::Span;

/// instantiates universal type variables introduced by generic parameters
/// with fresh inference variables
pub struct InstantiationFolder<'tcx> {
    tcx: TyCtx<'tcx>,
    substs: SubstsRef<'tcx>,
}

trait SubstsExt<'tcx> {
    fn forall(infcx: &InferCtx<'_, 'tcx>, forall: &Generics<'tcx>) -> SubstsRef<'tcx>;
}

impl<'tcx> SubstsExt<'tcx> for Substs<'tcx> {
    // creates a fresh inference variable for each type parameter in `forall`
    fn forall(infcx: &InferCtx<'_, 'tcx>, forall: &Generics<'tcx>) -> SubstsRef<'tcx> {
        let params = forall.params.iter().map(|p| infcx.new_infer_var(p.span));
        infcx.mk_substs(params)
    }
}

impl<'tcx> InstantiationFolder<'tcx> {
    pub fn new(infcx: &InferCtx<'_, 'tcx>, _span: Span, forall: &Generics<'tcx>) -> Self {
        let tcx = infcx.tcx;
        let substs = Substs::forall(infcx, forall);

        Self { tcx, substs }
    }
}

impl<'tcx> TypeFolder<'tcx> for InstantiationFolder<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        self.tcx
    }

    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        match ty.kind {
            TyKind::Param(param_ty) => self.substs[param_ty.idx.index()],
            _ => ty.inner_fold_with(self),
        }
    }
}

use crate::span::Span;
use crate::ty::{InferTy, List, Ty, TyKind, TypeFoldable, TypeFolder};
use crate::typeck::TyCtx;

crate trait Subst<'tcx>: Sized {
    fn subst_spanned(&self, tcx: TyCtx<'tcx>, substs: SubstRef<'tcx>, span: Option<Span>) -> Self;
    fn subst(&self, tcx: TyCtx<'tcx>, substs: SubstRef<'tcx>) -> Self {
        self.subst_spanned(tcx, substs, None)
    }
}

/// a substitution is simply a slice of `Ty`s, where the index of the Ty is the TyVid of the
/// inference variable.
/// this is compared for equality by pointer equality
/// i.e. the type for InferTy::TyVid(i) is Substitutions[i]
/// this is also used to represent a slice of `Ty`s
crate type SubstRef<'tcx> = &'tcx List<Ty<'tcx>>;

/// substitute inference variables according to some substitution
crate struct InferenceVarSubstFolder<'tcx> {
    tcx: TyCtx<'tcx>,
    substs: SubstRef<'tcx>,
}

impl<'tcx> InferenceVarSubstFolder<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, substs: SubstRef<'tcx>) -> Self {
        Self { tcx, substs }
    }
}

impl<'tcx> TypeFolder<'tcx> for InferenceVarSubstFolder<'tcx> {
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        match &ty.kind {
            &TyKind::Infer(InferTy::TyVar(tyvid)) => self.substs[tyvid.index as usize],
            _ => ty.inner_fold_with(self),
        }
    }

    fn tcx(&self) -> crate::typeck::TyCtx<'tcx> {
        self.tcx
    }
}

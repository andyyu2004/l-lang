use crate::span::Span;
use crate::ty::{InferTy, Ty, TyKind};
use crate::typeck::{List, TyCtx, TypeFoldable, TypeFolder};

crate trait Subst<'tcx>: Sized {
    fn subst_spanned(&self, tcx: TyCtx<'tcx>, substs: SubstRef<'tcx>, span: Option<Span>) -> Self;
    fn subst(&self, tcx: TyCtx<'tcx>, substs: SubstRef<'tcx>) -> Self {
        self.subst_spanned(tcx, substs, None)
    }
}

impl<'tcx, T> Subst<'tcx> for T
where
    T: TypeFoldable<'tcx>,
{
    fn subst_spanned(&self, tcx: TyCtx<'tcx>, substs: SubstRef<'tcx>, span: Option<Span>) -> Self {
        let mut folder = SubstFolder { tcx, substs };
        self.fold_with(&mut folder)
    }
}

/// a substitution is simply a slice of `Ty`s, where the index of the Ty is the TyVid of the
/// inference variable.
/// i.e. the type for InferTy::TyVid(i) is Substitutions[i]
crate type SubstRef<'tcx> = &'tcx List<Ty<'tcx>>;

crate struct SubstFolder<'tcx> {
    tcx: TyCtx<'tcx>,
    substs: SubstRef<'tcx>,
}

impl<'tcx> TypeFolder<'tcx> for SubstFolder<'tcx> {
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        match &ty.kind {
            &TyKind::Infer(InferTy::TyVar(tyvid)) => return self.substs[tyvid.index as usize],
            _ => ty.inner_fold_with(self),
        }
    }

    fn tcx(&self) -> crate::typeck::TyCtx<'tcx> {
        self.tcx
    }
}

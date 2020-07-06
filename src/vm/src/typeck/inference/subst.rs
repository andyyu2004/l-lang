use crate::typeck::{TyCtx, TypeFoldable, TypeFolder};
use crate::{span::Span, ty::Ty};

crate trait Subst<'tcx>: Sized {
    fn subst_spanned(&self, tcx: TyCtx<'tcx>, substs: &[Ty<'tcx>], span: Option<Span>) -> Self;
    fn subst(&self, tcx: TyCtx<'tcx>, substs: &[Ty<'tcx>]) -> Self {
        self.subst_spanned(tcx, substs, None)
    }
}

impl<'tcx, T> Subst<'tcx> for T
where
    T: TypeFoldable<'tcx>,
{
    fn subst_spanned(&self, tcx: TyCtx<'tcx>, substs: &[Ty<'tcx>], span: Option<Span>) -> Self {
        let mut folder = SubstFolder { tcx, substs };
        self.fold_with(&mut folder)
    }
}

/// a substitution is simply a slice of `Ty`s, where the index of the Ty is the TyVid of the
/// inference variable.
/// i.e. the type for InferTy::TyVid(i) is Substitutions[i]
crate type Substitutions<'tcx> = [Ty<'tcx>];

crate struct SubstFolder<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    substs: &'a Substitutions<'tcx>,
}

impl<'a, 'tcx> TypeFolder<'tcx> for SubstFolder<'a, 'tcx> {
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        ty.fold_with(self)
    }

    fn tcx(&self) -> crate::typeck::TyCtx<'tcx> {
        self.tcx
    }
}

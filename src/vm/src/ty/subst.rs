use crate::ir::{self, DefId};
use crate::span::Span;
use crate::ty::{Forall, InferTy, List, Ty, TyKind, TypeFoldable, TypeFolder};
use crate::typeck::{inference::InferCtx, TyCtx};
use indexed_vec::Idx;
use rustc_hash::FxHashMap;

pub trait Subst<'tcx>: Sized {
    fn subst(&self, tcx: TyCtx<'tcx>, substs: SubstRef<'tcx>) -> Self;
}

impl<'tcx, T> Subst<'tcx> for T
where
    T: TypeFoldable<'tcx>,
{
    fn subst(&self, tcx: TyCtx<'tcx>, substs: SubstRef<'tcx>) -> Self {
        let mut folder = InferenceVarSubstFolder { tcx, substs };
        self.fold_with(&mut folder)
    }
}

/// a substitution is simply a slice of `Ty`s, where the index of the Ty is the TyVid of the
/// inference variable.
/// this is compared for equality by pointer equality
/// i.e. the type for InferTy::TyVid(i) is Substitutions[i]
/// this is also used to represent a slice of `Ty`s
pub type SubstRef<'tcx> = &'tcx List<Ty<'tcx>>;

/// instantiates universal type variables with fresh inference variables
pub struct InstantiationFolder<'tcx> {
    tcx: TyCtx<'tcx>,
    substs: SubstRef<'tcx>,
}

impl<'tcx> InstantiationFolder<'tcx> {
    pub fn new(infcx: &InferCtx<'_, 'tcx>, forall: &Forall<'tcx>) -> Self {
        let tcx = infcx.tcx;
        // check that the index of the binders aren't weird
        let n = forall.binders.len();
        assert!(forall.binders.iter().map(|idx| idx.index()).eq(0..n));
        let substs = tcx.mk_substs((0..n).map(|_| infcx.new_infer_var()));
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

/// substitute inference variables according to some substitution
pub struct InferenceVarSubstFolder<'tcx> {
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
        match ty.kind {
            TyKind::Infer(InferTy::TyVar(tyvid)) => self.substs[tyvid.index as usize],
            _ => ty.inner_fold_with(self),
        }
    }

    fn tcx(&self) -> TyCtx<'tcx> {
        self.tcx
    }
}

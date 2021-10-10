use crate::*;
use ty::*;

pub trait Subst<'tcx>: Sized {
    /// replaces all the type parameters with the appropriate substitution
    fn subst(&self, tcx: TyCtx<'tcx>, substs: SubstsRef<'tcx>) -> Self;
}

impl<'tcx, T> Subst<'tcx> for T
where
    T: TypeFoldable<'tcx>,
{
    fn subst(&self, tcx: TyCtx<'tcx>, substs: SubstsRef<'tcx>) -> Self {
        let mut folder = SubstsFolder { tcx, substs };
        self.fold_with(&mut folder)
    }
}

pub struct SubstsFolder<'tcx> {
    tcx: TyCtx<'tcx>,
    substs: SubstsRef<'tcx>,
}

impl<'tcx> TypeFolder<'tcx> for SubstsFolder<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        self.tcx
    }

    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        if !ty.has_ty_params() {
            return ty;
        }

        match ty.kind {
            TyKind::Param(param_ty) => self.substs[param_ty.idx.index()],
            _ => ty.inner_fold_with(self),
        }
    }
}

/// a substitution is simply a slice of `Ty`s, where the index of the Ty is the TyVid of the
/// inference variable.
/// this is compared for equality by pointer equality
/// i.e. the type for `InferTy::TyVid(i)` is `substs[i]`
/// this is also often used to represent a slice/list of `Ty`s
pub type SubstsRef<'tcx> = &'tcx Substs<'tcx>;

// we require this indirection allow impl blocks
pub type Substs<'tcx> = List<Ty<'tcx>>;

impl<'tcx> Substs<'tcx> {
    /// crates an identity substitution given the generics for some item
    pub fn id_for_def(tcx: TyCtx<'tcx>, def_id: DefId) -> SubstsRef<'tcx> {
        let generics = tcx.generics_of(def_id);
        Self::id_for_generics(tcx, generics)
    }

    /// crates an identity substitution given the generics for some item
    fn id_for_generics(tcx: TyCtx<'tcx>, generics: &'tcx ty::Generics<'tcx>) -> SubstsRef<'tcx> {
        let params = generics.params.iter().map(|p| tcx.mk_ty_param(p.id.def, p.index, p.ident));
        tcx.mk_substs(params)
    }
}

/// substitute inference variables according to some substitution
pub struct InferVarSubstsFolder<'tcx> {
    tcx: TyCtx<'tcx>,
    substs: SubstsRef<'tcx>,
}

impl<'tcx> InferVarSubstsFolder<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, substs: SubstsRef<'tcx>) -> Self {
        Self { tcx, substs }
    }
}

impl<'tcx> TypeFolder<'tcx> for InferVarSubstsFolder<'tcx> {
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        if !ty.has_infer_vars() {
            return ty;
        }
        match ty.kind {
            TyKind::Infer(InferTy::TyVar(tyvid)) => self.substs[tyvid.index as usize],
            _ => ty.inner_fold_with(self),
        }
    }

    fn tcx(&self) -> TyCtx<'tcx> {
        self.tcx
    }
}

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
/// i.e. the type for `InferTy::TyVid(i)` is `Substitutions[i]`
/// this is also used to represent a slice of `Ty`s
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

impl<'tcx> TyCtx<'tcx> {
    /// returns a substitution that will turn `scheme` into `t`
    /// used for finding all the substitutions for monomorphization
    /// we can assume that these two types are unifiable as this should
    /// only be called after successful typechecking
    pub fn unify_scheme(self, def_id: DefId, t: Ty<'tcx>) -> SubstsRef<'tcx> {
        assert!(!t.has_infer_vars());
        todo!();
        // debug!("unify {} with {}", scheme, t);
        // let (generics, s) = scheme.unpack();
        // let substs = Substs::id_for_generics(self, generics);
        // TypeMatcher::new(self, substs).unify(s, t)
    }
}

struct TypeMatcher<'tcx> {
    tcx: TyCtx<'tcx>,
    substs: Vec<Ty<'tcx>>,
    // just to check we're not setting the same param to different types
    #[cfg(debug_assertions)]
    map: rustc_hash::FxHashMap<ParamTy, Ty<'tcx>>,
}

// if this pattern is used again, abstract into a trait
impl<'tcx> TypeMatcher<'tcx> {
    fn new(tcx: TyCtx<'tcx>, substs: SubstsRef<'tcx>) -> Self {
        Self {
            tcx,
            substs: substs.to_vec(),
            #[cfg(debug_assertions)]
            map: Default::default(),
        }
    }

    fn unify(&mut self, s: Ty<'tcx>, t: Ty<'tcx>) -> SubstsRef<'tcx> {
        self.unify_inner(s, t);
        self.tcx.intern_substs(&self.substs)
    }

    fn unify_inner(&mut self, s: Ty<'tcx>, t: Ty<'tcx>) {
        if s == t || !s.has_ty_params() {
            return;
        }
        match (s.kind, t.kind) {
            (ty::Param(p), _) => {
                // checks that if the param has already been set,
                // it is set to the same type as last time
                // even with debug_assert, the cfg is necessary
                #[cfg(debug_assertions)]
                debug_assert!(self.map.insert(p, t).map_or(true, |ty| ty == t));
                self.substs[p.idx.index()] = t;
            }
            (Box(t), Box(u)) => self.unify_inner(t, u),
            (Tuple(xs), Tuple(ys)) => self.match_tuples(xs, ys),
            (Array(t, m), Array(u, n)) if m == n => self.unify_inner(t, u),
            (Adt(adtx, substsx), Adt(adty, substsy)) if adtx == adty =>
                self.match_tuples(substsx, substsy),
            (FnDef(defx, substsx), FnDef(defy, substsy)) if defx == defy =>
                self.match_tuples(substsx, substsy),
            (_, ty::Never) | (ty::Never, _) => {}
            (FnPtr(f), FnPtr(g)) => {
                self.match_tuples(f.params, g.params);
                self.unify_inner(f.ret, g.ret);
            }
            _ => panic!("cannot match {} {}", s, t),
        }
    }

    fn match_tuples(&mut self, r: SubstsRef<'tcx>, s: SubstsRef<'tcx>) {
        assert!(r.len() == s.len());
        r.iter().zip(s).for_each(|(t, u)| self.unify_inner(t, u));
    }
}

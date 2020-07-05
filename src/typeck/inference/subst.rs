use super::{Constraint, ConstraintKind, Constraints, InferCtx};
use crate::{
    ty::Ty, typeck::{InferResult, TyCtx, TypeFoldable, TypeFolder}
};
use rustc_hash::FxHashMap;
use std::ptr;

#[derive(Deref, DerefMut, Debug)]
crate struct Subst<'tcx>(FxHashMap<u32, Ty<'tcx>>);

crate struct SubstFolder<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    subst: &'a Subst<'tcx>,
}

impl<'a, 'tcx> TypeFolder<'tcx> for SubstFolder<'a, 'tcx> {
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        ty.fold_with(self)
    }

    fn tcx(&self) -> crate::typeck::TyCtx<'tcx> {
        self.tcx
    }
}

impl<'tcx> Subst<'tcx> {
    pub fn null() -> Self {
        Self(Default::default())
    }
}

impl<'a, 'tcx> InferCtx<'a, 'tcx> {
    pub fn solve(&self, constraints: &Constraints<'tcx>) -> InferResult<Subst<'tcx>> {
        constraints.iter().fold(Ok(Subst::null()), |acc, c| {
            let subst = self.solve_constraint(c)?;
            acc
        })
    }

    fn compose_subst(&self, s: Subst<'tcx>, t: Subst<'tcx>) -> Subst<'tcx> {
        todo!()
    }

    fn solve_constraint(&self, constraint: &Constraint<'tcx>) -> InferResult<Subst<'tcx>> {
        match constraint.kind {
            ConstraintKind::Eq(s, t) => self.unify(constraint, s, t),
        }
    }

    fn unify(
        &self,
        constraint: &Constraint<'tcx>,
        s: Ty<'tcx>,
        t: Ty<'tcx>,
    ) -> InferResult<Subst<'tcx>> {
        match (s, t) {
            // we can use ptr eq for type equality as each type is allocated exactly once
            (_, _) if ptr::eq(s, t) => Ok(Subst::null()),
            (x, y) => panic!("unification error {} != {}", x, y),
        }
    }
}

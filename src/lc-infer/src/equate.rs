use crate::At;
use lc_core::ty::{self, Ty, TyCtx, TypeRelation, TypeResult};
use std::ops::Deref;

pub struct Equate<'a, 'tcx> {
    pub at: &'a At<'a, 'tcx>,
}

impl<'a, 'tcx> TypeRelation<'tcx> for Equate<'a, 'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        self.tcx
    }

    fn relate_tys(&mut self, a: Ty<'tcx>, b: Ty<'tcx>) -> TypeResult<'tcx, Ty<'tcx>> {
        let mut inner = self.inner.borrow_mut();
        let mut type_vars = inner.type_variables();

        let a = type_vars.instantiate_if_known(a);
        let b = type_vars.instantiate_if_known(b);

        match (&a.kind, &b.kind) {
            _ if a == b => {}
            (&ty::Infer(ty::TyVar(a_id)), &ty::Infer(ty::TyVar(b_id))) =>
                type_vars.equate(a_id, b_id),
            (&ty::Infer(ty::TyVar(vid)), _) => type_vars.instantiate(vid, b)?,
            (_, &ty::Infer(ty::TyVar(vid))) => type_vars.instantiate(vid, a)?,
            (ty::Error, _) | (_, ty::Error) => return Ok(self.infcx.set_ty_err()),
            _ => {
                // drop the refcell borrow so the recursive call doesn't panic
                drop(inner);
                self.relate_inner(a, b)?;
            }
        };
        Ok(a)
    }
}

impl<'a, 'tcx> Deref for Equate<'a, 'tcx> {
    type Target = At<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        self.at
    }
}

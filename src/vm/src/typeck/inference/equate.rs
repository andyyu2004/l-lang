use super::At;
use crate::typeck::{TyCtx, TypeRelation};
use crate::{
    error::{TypeError, TypeResult}, ty::{self, Ty}
};

crate struct Equate<'a, 'tcx> {
    pub at: &'a At<'a, 'tcx>,
}

impl<'a, 'tcx> TypeRelation<'tcx> for Equate<'a, 'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        self.at.infcx.tcx
    }

    fn relate_tys(&mut self, a: Ty<'tcx>, b: Ty<'tcx>) -> TypeResult<'tcx, Ty<'tcx>> {
        // todo keep the span somewhere
        let infcx = self.at.infcx;
        let mut inner = infcx.inner.borrow_mut();
        let mut type_vars = inner.type_variables();

        match (&a.kind, &b.kind) {
            (&ty::Infer(ty::TyVar(a_id)), &ty::Infer(ty::TyVar(b_id))) => {
                type_vars.equate(a_id, b_id)
            }
            (&ty::Infer(ty::TyVar(vid)), _) => type_vars.instantiate(vid, b),
            (_, &ty::Infer(ty::TyVar(vid))) => type_vars.instantiate(vid, a),
            _ if a == b => Ok(()),
            _ => Err(TypeError::UnificationFailure(a, b)),
        }?;
        Ok(a)
    }
}

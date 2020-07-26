use super::At;
use crate::error::{TypeError, TypeResult};
use crate::ty::*;
use crate::typeck::{TyCtx, TypeRelation};

crate struct Equate<'a, 'tcx> {
    pub at: &'a At<'a, 'tcx>,
}

impl<'a, 'tcx> TypeRelation<'tcx> for Equate<'a, 'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        self.at.infcx.tcx
    }

    fn relate_tys(&mut self, a: Ty<'tcx>, b: Ty<'tcx>) -> TypeResult<'tcx, Ty<'tcx>> {
        let infcx = self.at.infcx;
        let mut inner = infcx.inner.borrow_mut();
        let mut type_vars = inner.type_variables();

        let a = type_vars.instantiate_if_known(a);
        let b = type_vars.instantiate_if_known(b);
        dbg!(a);
        dbg!(b);

        match (&a.kind, &b.kind) {
            _ if a == b => {}
            (&Infer(TyVar(a_id)), &Infer(TyVar(b_id))) => type_vars.equate(a_id, b_id),
            (&Infer(TyVar(vid)), _) => type_vars.instantiate(vid, b),
            (_, &Infer(TyVar(vid))) => type_vars.instantiate(vid, a),
            _ => {
                drop(inner);
                self.relate_inner(a, b)?;
            }
        };
        Ok(a)
    }
}

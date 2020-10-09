use crate::TyConv;
use lcore::ty::*;

pub trait Typeof<'tcx> {
    fn ty(&self, tcx: TyCtx<'tcx>, substs: SubstsRef<'tcx>) -> Ty<'tcx>;
}

impl<'tcx> Typeof<'tcx> for FieldTy<'tcx> {
    /// return type of the field
    // we require this indirection instead of storing `ty: Ty` directly as a field
    // because fields may refer to the the struct/enum that it is declared in
    // therefore, the lowering must be done post type collection
    fn ty(&self, tcx: TyCtx<'tcx>, substs: SubstsRef<'tcx>) -> Ty<'tcx> {
        // TODO cache this result somewhere?
        let ty = tcx.ir_ty_to_ty(&self.ir_ty);
        ty.subst(tcx, substs)
    }
}

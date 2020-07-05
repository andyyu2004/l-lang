use super::TyCtx;
use crate::ty::{Ty, TyKind};

crate trait TypeFoldable<'tcx> {
    fn fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>;
}

impl<'tcx> TypeFoldable<'tcx> for Ty<'tcx> {
    fn fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        match self.kind {
            TyKind::_Phantom(_) => return self,
            TyKind::Unit | TyKind::Char | TyKind::Num | TyKind::Bool => return self,
        }
    }
}

crate trait TypeFolder<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx>;
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx>;
}

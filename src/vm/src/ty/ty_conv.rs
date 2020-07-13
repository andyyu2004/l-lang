//! conversion of ir::Ty to ty::Ty

use crate::ir;
use crate::ty::{Ty, TyKind};
use crate::typeck::TyCtx;

crate trait TyConv<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx>;
}

impl<'a, 'tcx> dyn TyConv<'tcx> + 'a {
    pub fn ir_ty_to_ty(&self, ir_ty: &ir::Ty) -> Ty<'tcx> {
        let tcx = self.tcx();
        match &ir_ty.kind {
            ir::TyKind::Array(ty) => tcx.mk_ty(TyKind::Array(self.ir_ty_to_ty(ty))),
            ir::TyKind::Path(path) => match path.res {
                ir::Res::PrimTy(prim_ty) => tcx.mk_prim_ty(prim_ty),
                _ => panic!("unexpected resolution"),
            },
        }
    }
}

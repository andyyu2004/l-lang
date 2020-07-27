//! conversion of ir::Ty to ty::Ty

use crate::ir;
use crate::span::Span;
use crate::ty::{Ty, TyKind};
use crate::typeck::TyCtx;

crate trait TyConv<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx>;
    fn infer_ty(&self, span: Span) -> Ty<'tcx>;
}

impl<'a, 'tcx> dyn TyConv<'tcx> + 'a {
    /// lower `ir::Ty` to `ty::Ty`
    pub fn ir_ty_to_ty(&self, ir_ty: &ir::Ty) -> Ty<'tcx> {
        let tcx = self.tcx();
        match &ir_ty.kind {
            ir::TyKind::Array(ty) => tcx.mk_ty(TyKind::Array(self.ir_ty_to_ty(ty))),
            ir::TyKind::Path(path) => match path.res {
                ir::Res::PrimTy(prim_ty) => tcx.mk_prim_ty(prim_ty),
                ir::Res::Def(def_id, def_kind) => match def_kind {
                    ir::DefKind::TyParam => tcx.mk_ty_param(def_id),
                    ir::DefKind::Fn => panic!(),
                },
                ir::Res::Local(_) => panic!("unexpected resolution"),
            },
            ir::TyKind::Tuple(tys) => tcx.mk_tup(tys.iter().map(|ty| self.ir_ty_to_ty(ty))),
            ir::TyKind::Infer => self.infer_ty(ir_ty.span),
            ir::TyKind::Fn(params, ret) => tcx.mk_ty(TyKind::Fn(
                tcx.mk_substs(params.iter().map(|ty| self.ir_ty_to_ty(ty))),
                ret.map(|ty| self.ir_ty_to_ty(ty)).unwrap_or(tcx.types.unit),
            )),
        }
    }
}

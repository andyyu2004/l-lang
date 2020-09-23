//! conversion of ir::Ty to ty::Ty

use crate::ir::{self, Res};
use crate::span::Span;
use crate::ty::{Ty, TyKind};
use crate::typeck::TyCtx;

pub trait TyConv<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx>;
    fn infer_ty(&self, span: Span) -> Ty<'tcx>;
}

impl<'a, 'tcx> dyn TyConv<'tcx> + 'a {
    /// lower `ir::Ty` to `ty::Ty`
    pub fn ir_ty_to_ty(&self, ir_ty: &ir::Ty) -> Ty<'tcx> {
        let tcx = self.tcx();
        match &ir_ty.kind {
            ir::TyKind::Array(ty) => {
                // tcx.mk_array_ty(self.ir_ty_to_ty(ty), todo!()),
                todo!();
            }
            ir::TyKind::Path(path) => self.res_to_ty(path.res),
            ir::TyKind::Tuple(tys) => tcx.mk_tup_iter(tys.iter().map(|ty| self.ir_ty_to_ty(ty))),
            ir::TyKind::Infer => self.infer_ty(ir_ty.span),
            ir::TyKind::Ptr(m, ty) => tcx.mk_ptr_ty(*m, self.ir_ty_to_ty(ty)),
            ir::TyKind::Fn(params, ret) => tcx.mk_fn_ty(
                tcx.mk_substs(params.iter().map(|ty| self.ir_ty_to_ty(ty))),
                ret.map(|ty| self.ir_ty_to_ty(ty)).unwrap_or(tcx.types.unit),
            ),
        }
    }

    pub fn res_to_ty(&self, res: Res) -> Ty<'tcx> {
        let tcx = self.tcx();
        match res {
            ir::Res::PrimTy(prim_ty) => tcx.mk_prim_ty(prim_ty),
            ir::Res::Def(def_id, def_kind) => match def_kind {
                ir::DefKind::TyParam(idx) => tcx.mk_ty_param(def_id, idx),
                ir::DefKind::Struct => {
                    // TODO unsure how to deal with the forall currently
                    // instantiation requires an inferctx which may not be available if we are only
                    // performing type collection
                    let (_forall, ty) = tcx.collected_ty(def_id).expect_scheme();
                    ty
                }
                ir::DefKind::Ctor(..) => todo!(),
                ir::DefKind::Impl | ir::DefKind::Fn | ir::DefKind::Enum => unreachable!(),
            },
            ir::Res::SelfTy => todo!(),
            ir::Res::Err => tcx.mk_ty_err(),
            ir::Res::Local(_) => panic!("unexpected resolution"),
        }
    }

    pub fn fn_sig_to_ty(&self, sig: &ir::FnSig) -> Ty<'tcx> {
        let tcx = self.tcx();
        // None return type on fn sig implies unit type
        let ret_ty = sig.output.map(|ty| self.ir_ty_to_ty(ty)).unwrap_or(tcx.types.unit);
        let inputs = sig.inputs.iter().map(|ty| self.ir_ty_to_ty(ty));
        let input_tys = tcx.mk_substs(inputs);
        tcx.mk_fn_ty(input_tys, ret_ty)
    }
}

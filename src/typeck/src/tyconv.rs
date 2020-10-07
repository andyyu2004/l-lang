//! conversion of ir::Ty to ty::Ty

use lcore::ty::{Generics, Subst, Ty, TyParam, TypeError};
use lcore::TyCtx;
use span::Span;

pub trait TyConv<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx>;

    fn infer_ty(&self, span: Span) -> Ty<'tcx>;

    fn ir_ty_to_ty(&self, ir_ty: &ir::Ty) -> Ty<'tcx> {
        let tcx = self.tcx();
        match &ir_ty.kind {
            ir::TyKind::Array(_ty) => {
                // tcx.mk_array_ty(self.ir_ty_to_ty(ty), todo!()),
                todo!();
            }
            ir::TyKind::Path(path) => self.path_to_ty(path),
            ir::TyKind::Tuple(tys) => tcx.mk_tup_iter(tys.iter().map(|ty| self.ir_ty_to_ty(ty))),
            ir::TyKind::Infer => self.infer_ty(ir_ty.span),
            ir::TyKind::Ptr(m, ty) => tcx.mk_ptr_ty(*m, self.ir_ty_to_ty(ty)),
            ir::TyKind::Fn(params, ret) => tcx.mk_fn_ty(
                tcx.mk_substs(params.iter().map(|ty| self.ir_ty_to_ty(ty))),
                ret.map(|ty| self.ir_ty_to_ty(ty)).unwrap_or(tcx.types.unit),
            ),
        }
    }

    fn path_to_ty(&self, path: &ir::Path) -> Ty<'tcx> {
        let tcx = self.tcx();
        match path.res {
            ir::Res::PrimTy(prim_ty) => tcx.mk_prim_ty(prim_ty),
            ir::Res::Def(def_id, def_kind) => match def_kind {
                ir::DefKind::TyParam(idx) => tcx.mk_ty_param(def_id, idx),
                ir::DefKind::Struct | ir::DefKind::Enum => {
                    let expected_argc = tcx.resolutions.generic_arg_counts[&def_id];
                    // TODO assume for now only the last path segment has generic args
                    // this may not always be true e.g.
                    // ADT<T, U>::method<V>();
                    let generic_args = path.segments.last().unwrap().args;
                    // replace each generic parameter with either an inference variable
                    // or the specified type
                    let substs = match generic_args {
                        Some(args) =>
                            if args.args.len() != expected_argc {
                                let err =
                                    TypeError::GenericArgCount(expected_argc, args.args.len());
                                tcx.sess.build_error(path.span, err).emit();
                                return tcx.mk_ty_err();
                            } else {
                                tcx.mk_substs(args.args.iter().map(|ty| self.ir_ty_to_ty(ty)))
                            },
                        None => tcx.mk_substs((0..expected_argc).map(|_| self.infer_ty(path.span))),
                    };
                    let (_forall, ty) = tcx.collected_ty(def_id).expect_scheme();
                    ty.subst(tcx, substs)
                }
                ir::DefKind::Ctor(..) => todo!(),
                ir::DefKind::AssocFn | ir::DefKind::Impl | ir::DefKind::Fn => todo!(),
                ir::DefKind::Extern => todo!(),
            },
            ir::Res::SelfTy => todo!(),
            ir::Res::Err => tcx.mk_ty_err(),
            ir::Res::Local(_) => panic!("unexpected resolution"),
        }
    }

    fn lower_generics(&self, generics: &ir::Generics) -> Generics<'tcx> {
        let params =
            generics.params.iter().map(|&ir::TyParam { id, index, ident, span, default }| {
                TyParam { id, span, ident, index, default: default.map(|ty| self.ir_ty_to_ty(ty)) }
            });
        Generics { params: self.tcx().alloc_iter(params) }
    }

    fn fn_sig_to_ty(&self, sig: &ir::FnSig) -> Ty<'tcx> {
        let tcx = self.tcx();
        // None return type on fn sig implies unit type
        let ret_ty = sig.output.map(|ty| self.ir_ty_to_ty(ty)).unwrap_or(tcx.types.unit);
        let inputs = sig.inputs.iter().map(|ty| self.ir_ty_to_ty(ty));
        let input_tys = tcx.mk_substs(inputs);
        tcx.mk_fn_ty(input_tys, ret_ty)
    }
}

impl<'tcx> TyConv<'tcx> for TyCtx<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        *self
    }

    fn infer_ty(&self, _span: Span) -> Ty<'tcx> {
        panic!("tyctx can't lower types with inference variables")
    }
}

//! each conversion is parameterized by an instance of `TyConv`
//! there are two primary lowering contexts `TyCtx` and `InferCtx`
//! `InferCtx` allows inference variables, one does not

use ir::{DefId, DefKind, QPath, Res};
use lcore::ty::{FnSig, Generics, Subst, Substs, Ty, TyCtx, TyParam, TypeError};
use span::Span;

/// refer to module comments
pub trait TyConv<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx>;

    /// new inference variable
    fn infer_ty(&self, span: Span) -> Ty<'tcx>;

    /// whether or not inference variable are allowed
    /// if not, then every type should be explicitly stated
    /// e.g. this is false for parameter lists
    fn allow_infer(&self) -> bool;

    fn ir_ty_to_ty(&self, ir_ty: &ir::Ty<'tcx>) -> Ty<'tcx> {
        let tcx = self.tcx();
        match &ir_ty.kind {
            ir::TyKind::Box(ty) => tcx.mk_box_ty(self.ir_ty_to_ty(ty)),
            ir::TyKind::Fn(params, ret) => tcx.mk_fn_ptr(FnSig {
                params: tcx.mk_substs(params.iter().map(|ty| self.ir_ty_to_ty(ty))),
                ret: ret.map(|ty| self.ir_ty_to_ty(ty)).unwrap_or(tcx.types.unit),
            }),
            ir::TyKind::Path(qpath) => self.qpath_to_ty(qpath),
            ir::TyKind::Tuple(tys) => tcx.mk_tup_iter(tys.iter().map(|ty| self.ir_ty_to_ty(ty))),
            ir::TyKind::Ptr(ty) => tcx.mk_ptr_ty(self.ir_ty_to_ty(ty)),
            ir::TyKind::Array(_ty) => {
                // tcx.mk_array_ty(self.ir_ty_to_ty(ty), todo!()),
                todo!();
            }
            ir::TyKind::Infer => self.infer_ty(ir_ty.span),
            ir::TyKind::Err => tcx.mk_ty_err(),
        }
    }

    fn qpath_to_ty(&self, qpath: &ir::QPath<'tcx>) -> Ty<'tcx> {
        match qpath {
            QPath::Resolved(path) => self.path_to_ty(path),
            QPath::TypeRelative(_, _) => todo!(),
        }
    }

    fn def_to_ty(&self, path: &ir::Path<'tcx>, def_id: DefId, def_kind: DefKind) -> Ty<'tcx> {
        let tcx = self.tcx();
        match def_kind {
            DefKind::TyParam(idx) => tcx.mk_ty_param(def_id, idx, tcx.defs().ident(def_id)),
            DefKind::Struct | DefKind::Enum | DefKind::TypeAlias => {
                let expected_argc = tcx.generics_of(def_id).params.len();
                // there should only be generic args in the very last position.
                // the preceding segments should be a module path
                // the segments afterwards are type relative
                let (last, segs) = path.segments.split_last().unwrap();
                self.ensure_no_generic_args(segs);
                let generic_args = last.args;

                let emit_err = |argc, err| {
                    tcx.sess
                        .build_error(path.span, err)
                        .labelled_span(
                            tcx.defs().generics(def_id).span,
                            format!(
                                "{} generic parameter{} declared here",
                                expected_argc,
                                pluralize!(expected_argc)
                            ),
                        )
                        .labelled_span(
                            generic_args.map(|args| args.span).unwrap_or(last.ident.span),
                            format!(
                                "but {} generic argument{} provided here",
                                argc,
                                pluralize!(argc)
                            ),
                        )
                        .emit();
                    tcx.mk_ty_err()
                };

                // replace each generic parameter with either the specified
                // type argument or id generics
                let substs = match generic_args {
                    Some(args) => {
                        let argc = args.args.len();
                        if argc != expected_argc {
                            return emit_err(
                                argc,
                                TypeError::GenericArgCount(expected_argc, args.args.len()),
                            );
                        } else {
                            tcx.mk_substs(args.args.iter().map(|ty| self.ir_ty_to_ty(ty)))
                        }
                    }
                    // TODO this case below is probably not correct
                    None if self.allow_infer() => Substs::id_for_def(tcx, def_id),
                    None if expected_argc == 0 => Substs::empty(),
                    None => return emit_err(0, TypeError::GenericArgCount(expected_argc, 0)),
                };
                let ty = tcx.type_of(def_id);
                ty.subst(tcx, substs)
            }
            DefKind::Ctor(..) | DefKind::Trait | DefKind::Fn | DefKind::AssocFn | DefKind::Impl =>
                todo!(),
            DefKind::Macro | DefKind::Mod | DefKind::Extern | DefKind::Use =>
                unreachable!("unexpected defkind `{}`", def_kind),
        }
    }

    fn path_to_ty(&self, path: &ir::Path<'tcx>) -> Ty<'tcx> {
        let tcx = self.tcx();
        match path.res {
            Res::PrimTy(prim_ty) => tcx.mk_prim_ty(prim_ty),
            Res::Def(def_id, def_kind) => self.def_to_ty(path, def_id, def_kind),
            Res::SelfTy { impl_def } => tcx.type_of(impl_def),
            Res::Local(..) | Res::SelfVal { .. } => panic!("unexpected resolution"),
            Res::Err => tcx.mk_ty_err(),
        }
    }

    fn ensure_no_generic_args(&self, segments: &[ir::PathSegment<'tcx>]) {
        segments.iter().for_each(|segment| assert!(segment.args.is_none()))
    }

    fn lower_generics(&self, generics: &ir::Generics<'tcx>) -> &'tcx Generics<'tcx> {
        let tcx = self.tcx();
        let params =
            generics.params.iter().map(|&ir::TyParam { id, index, ident, span, default }| {
                TyParam { id, span, ident, index, default: default.map(|ty| self.ir_ty_to_ty(ty)) }
            });
        tcx.alloc(Generics { params: tcx.alloc_iter(params) })
    }

    fn ir_fn_sig_to_ty(&self, sig: &ir::FnSig<'tcx>) -> Ty<'tcx> {
        self.tcx().mk_fn_ptr(self.lower_fn_sig(sig))
    }

    fn lower_fn_sig(&self, sig: &ir::FnSig<'tcx>) -> FnSig<'tcx> {
        let tcx = self.tcx();
        let params = tcx.mk_substs(sig.inputs.iter().map(|ty| self.ir_ty_to_ty(ty)));
        // `None` return type on fn sig implies unit type
        let ret = sig.output.map(|ty| self.ir_ty_to_ty(ty)).unwrap_or(tcx.types.unit);
        FnSig { params, ret }
    }
}

impl<'tcx> TyConv<'tcx> for TyCtx<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        *self
    }

    fn infer_ty(&self, _span: Span) -> Ty<'tcx> {
        panic!("tyctx can't lower types with inference variables")
    }

    fn allow_infer(&self) -> bool {
        false
    }
}

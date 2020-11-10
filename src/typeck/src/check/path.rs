use crate::{FnCtx, TyConv};
use ir::{CtorKind, DefId, DefKind, QPath, Res};
use lcore::ty::*;
use span::Span;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    crate fn resolve_qpath(&mut self, qpath: &QPath<'tcx>) -> Res {
        match qpath {
            QPath::Resolved(path) => path.res,
            QPath::TypeRelative(ty, segment) =>
                self.resolve_type_relative_path(qpath.span(), self.ir_ty_to_ty(ty), segment),
        }
    }

    crate fn check_struct_path(
        &mut self,
        xpat: &impl ir::ExprOrPat<'tcx>,
        qpath: &ir::QPath<'tcx>,
    ) -> Option<(&'tcx VariantTy<'tcx>, Ty<'tcx>)> {
        let ty = self.check_qpath(xpat, qpath);
        let res = self.resolve_qpath(qpath);
        // we don't directly return `substs` as it can be accessed through `ty`
        let variant = match res {
            Res::Def(_, DefKind::Struct) => match ty.kind {
                Adt(adt, _substs) => Some((adt.single_variant(), ty)),
                _ => unreachable!(),
            },
            Res::SelfVal { impl_def } => {
                let self_ty = self.type_of(impl_def);
                assert_eq!(self_ty, ty);
                match self_ty.kind {
                    Adt(adt, _substs) => Some((adt.single_variant(), self_ty)),
                    _ => unreachable!(),
                }
            }
            Res::Def(def_id, DefKind::Ctor(CtorKind::Struct)) => match ty.kind {
                Adt(adt, _substs) => Some((adt.variant_with_ctor(def_id), ty)),
                _ => unreachable!(),
            },
            Res::Local(_) => None,
            Res::PrimTy(..) | ir::Res::SelfTy { .. } => unreachable!(),
            _ => unimplemented!("{} (res: {:?})", qpath, res),
        };

        variant.or_else(|| {
            self.emit_ty_err(
                qpath.span(),
                TypeError::Msg(format!("expected struct path, found {:?}", qpath)),
            );
            None
        })
    }

    crate fn check_qpath(
        &mut self,
        xpat: &impl ir::ExprOrPat<'tcx>,
        qpath: &QPath<'tcx>,
    ) -> Ty<'tcx> {
        match qpath {
            QPath::Resolved(path) => self.check_expr_path(xpat, path),
            QPath::TypeRelative(self_ty, segment) =>
                self.check_type_relative_path(xpat, qpath, self.ir_ty_to_ty(self_ty), segment),
        }
    }

    crate fn check_type_relative_path(
        &mut self,
        xpat: &impl ir::ExprOrPat<'tcx>,
        qpath: &QPath<'tcx>,
        self_ty: Ty<'tcx>,
        segment: &ir::PathSegment<'tcx>,
    ) -> Ty<'tcx> {
        let res = self.resolve_type_relative_path(xpat.span(), self_ty, segment);
        self.record_type_relative_res(xpat.id(), res);
        let (def_id, def_kind) = res.expect_def();
        self.check_res_def_with_partial_substs(xpat, qpath.span(), def_id, def_kind)
    }

    crate fn check_expr_path(
        &mut self,
        xpat: &impl ir::ExprOrPat<'tcx>,
        path: &ir::Path,
    ) -> Ty<'tcx> {
        self.check_res(xpat, path.span, path.res)
    }

    fn check_res(&mut self, xpat: &impl ir::ExprOrPat<'tcx>, span: Span, res: Res) -> Ty<'tcx> {
        match res {
            Res::Local(id) => self.local_ty(id).ty,
            Res::Def(def_id, def_kind) => self.check_res_def(xpat, span, def_id, def_kind),
            Res::PrimTy(_) => panic!("found type resolution in value namespace"),
            Res::SelfVal { impl_def } => self.type_of(impl_def),
            Res::SelfTy { .. } => todo!(),
            Res::Err => self.set_ty_err(),
        }
    }

    fn check_res_def_with_partial_substs(
        &mut self,
        xpat: &impl ir::ExprOrPat<'tcx>,
        span: Span,
        def_id: DefId,
        def_kind: DefKind,
    ) -> Ty<'tcx> {
        match def_kind {
            // instantiate type parameters with a fresh substitution
            DefKind::Fn
            | DefKind::AssocFn
            | DefKind::Enum
            | DefKind::Struct
            | DefKind::Ctor(..) => self.instatiate(xpat, span, def_id),
            DefKind::TyParam(..) | DefKind::Extern | DefKind::Impl => unreachable!(),
        }
    }

    fn check_res_def(
        &mut self,
        xpat: &impl ir::ExprOrPat<'tcx>,
        span: Span,
        def_id: DefId,
        def_kind: DefKind,
    ) -> Ty<'tcx> {
        self.check_res_def_with_partial_substs(xpat, span, def_id, def_kind)
    }
}

use crate::{FnCtx, TyConv};
use ir::{CtorKind, DefId, DefKind, QPath, Res};
use lc_core::ty::*;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub(crate) fn check_struct_path(
        &mut self,
        xpat: &dyn ir::ExprOrPat<'tcx>,
        qpath: &ir::QPath<'tcx>,
    ) -> Option<(&'tcx VariantTy, Ty<'tcx>)> {
        let (res, ty) = self.resolve_qpath(xpat, qpath);
        // we don't directly return `substs` as it can be accessed through `ty`
        let variant = match res {
            Res::Def(_, DefKind::Struct | DefKind::TypeAlias) | Res::SelfVal { .. } =>
                match ty.kind {
                    Adt(adt, _substs) => Some((adt.single_variant(), ty)),
                    _ => unreachable!(),
                },
            Res::Def(def_id, DefKind::Ctor(CtorKind::Struct)) => match ty.kind {
                Adt(adt, _substs) => Some((adt.variant_with_ctor(def_id), ty)),
                _ => unreachable!(),
            },
            Res::Local(..) => None,
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

    pub(crate) fn check_qpath(
        &mut self,
        xpat: &dyn ir::ExprOrPat<'tcx>,
        qpath: &QPath<'tcx>,
    ) -> Ty<'tcx> {
        let (_res, ty) = self.resolve_qpath(xpat, qpath);
        ty
    }

    pub(crate) fn resolve_qpath(
        &mut self,
        xpat: &dyn ir::ExprOrPat<'tcx>,
        qpath: &QPath<'tcx>,
    ) -> (Res, Ty<'tcx>) {
        match qpath {
            QPath::Resolved(path) => (path.res, self.check_expr_path(xpat, path)),
            QPath::TypeRelative(self_ty, segment) =>
                self.check_type_relative_path(xpat, self.ir_ty_to_ty(self_ty), segment),
        }
    }

    pub(crate) fn check_type_relative_path(
        &mut self,
        xpat: &dyn ir::ExprOrPat<'tcx>,
        self_ty: Ty<'tcx>,
        segment: &ir::PathSegment<'tcx>,
    ) -> (Res, Ty<'tcx>) {
        let res = self.resolve_type_relative_path(xpat, self_ty, segment);
        if let Res::Err = res {
            return (res, self.mk_ty_err());
        }
        self.record_type_relative_res(xpat.id(), res);
        let (def_id, def_kind) = res.expect_def();
        let ty = self.check_res_def(xpat, def_id, def_kind);
        (res, ty)
    }

    pub(crate) fn check_expr_path(
        &mut self,
        xpat: &dyn ir::ExprOrPat<'tcx>,
        path: &ir::Path,
    ) -> Ty<'tcx> {
        self.check_res(xpat, path.res)
    }

    fn check_res(&mut self, xpat: &dyn ir::ExprOrPat<'tcx>, res: Res) -> Ty<'tcx> {
        match res {
            Res::Local(id) => self.local_ty(id).ty,
            Res::Def(def_id, def_kind) => self.check_res_def(xpat, def_id, def_kind),
            Res::PrimTy(..) => panic!("found type resolution in value namespace"),
            Res::SelfVal { impl_def } => self.type_of(impl_def),
            Res::SelfTy { .. } => todo!(),
            Res::Err => self.set_ty_err(),
        }
    }

    fn check_res_def(
        &mut self,
        xpat: &dyn ir::ExprOrPat<'tcx>,
        def_id: DefId,
        def_kind: DefKind,
    ) -> Ty<'tcx> {
        match def_kind {
            // instantiate type parameters with a fresh substitution
            DefKind::Ctor(..)
            | DefKind::Fn
            | DefKind::AssocFn
            | DefKind::Enum
            | DefKind::TypeAlias
            | DefKind::Struct => self.instantiate(xpat, def_id),
            DefKind::Trait => todo!(),
            DefKind::TyParam(..)
            | DefKind::Macro
            | DefKind::Impl
            | DefKind::Use
            | DefKind::Mod
            | DefKind::Extern => unreachable!(),
        }
    }
}

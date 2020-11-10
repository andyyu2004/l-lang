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
            QPath::Resolved(path) => self.check_expr_path(path),
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
        // these substitutions are used to partially initialize
        // the type scheme of the resolved type
        // consider the following example
        // struct S<T> { t: T }
        // impl<T> S<T> {
        //     fn new(t: T) -> Self {
        //         Self { t }
        //     }
        // }
        // fn main() { S::new(5) }
        //
        // S::new will resolve to a type relative path on struct S
        // in tyconv, we will implicitly create a new inference variable on S,
        // S::<?0>::new
        //
        // S::new will resolve to the appropriate function
        // however, if we instantiate S::new with entirely fresh inference variables like normal,
        // it loses its connection to ?0 and we will have a false `type annotations required`
        // error (on ?0 and S::new)
        //
        // the current solution to this is to pass a "partial substitution" so we
        // instantiate the type scheme of `S::new` with ?0 instead of some new variable ?1
        let substs = match self_ty.kind {
            Adt(_, substs) => substs,
            _ => todo!(),
        };
        self.check_res_def_with_partial_substs(qpath.span(), def_id, def_kind, substs)
    }

    crate fn check_expr_path(&mut self, path: &ir::Path) -> Ty<'tcx> {
        self.check_res(path.span, path.res)
    }

    fn check_res(&mut self, span: Span, res: Res) -> Ty<'tcx> {
        match res {
            Res::Local(id) => self.local_ty(id).ty,
            Res::Def(def_id, def_kind) => self.check_res_def(span, def_id, def_kind),
            Res::SelfVal { impl_def } => self.type_of(impl_def),
            Res::SelfTy { .. } => todo!(),
            Res::PrimTy(_) => panic!("found type resolution in value namespace"),
            Res::Err => self.set_ty_err(),
        }
    }

    fn check_res_def_with_partial_substs(
        &mut self,
        span: Span,
        def_id: DefId,
        def_kind: DefKind,
        partial_substs: SubstsRef<'tcx>,
    ) -> Ty<'tcx> {
        match def_kind {
            // instantiate ty params
            DefKind::Fn
            | DefKind::AssocFn
            | DefKind::Enum
            | DefKind::Struct
            | DefKind::Ctor(..) => self.instantiate(span, self.type_of(def_id), partial_substs),
            DefKind::TyParam(..) | DefKind::Extern | DefKind::Impl => unreachable!(),
        }
    }

    fn check_res_def(&mut self, span: Span, def_id: DefId, def_kind: DefKind) -> Ty<'tcx> {
        self.check_res_def_with_partial_substs(span, def_id, def_kind, Substs::empty())
    }
}

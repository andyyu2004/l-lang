use super::FnCtx;
use ast::Ident;
use bimap::BiMap;
use ir::{self, CtorKind, DefKind, Res};
use lcore::ty::*;
use rustc_hash::FxHashMap;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// typechecks a given pattern with its expected type
    pub fn check_pat(&mut self, pat: &ir::Pattern<'tcx>, ty: Ty<'tcx>) -> Ty<'tcx> {
        let pat_ty = match &pat.kind {
            ir::PatternKind::Box(inner) => self.check_pat_box(pat, inner, ty),
            ir::PatternKind::Binding(_, _, mtbl) => self.def_local(pat.id, *mtbl, ty),
            ir::PatternKind::Tuple(pats) => self.check_pat_tuple(pat, pats, ty),
            ir::PatternKind::Lit(expr) => self.check_pat_lit(expr, ty),
            ir::PatternKind::Variant(qpath, pats) => self.check_pat_variant(pat, qpath, pats, ty),
            ir::PatternKind::Path(qpath) => self.check_pat_path(pat, qpath, ty),
            ir::PatternKind::Struct(qpath, fields) => self.check_pat_struct(pat, qpath, fields, ty),
            ir::PatternKind::Wildcard => ty,
        };
        self.record_ty(pat.id, pat_ty)
    }

    fn check_pat_struct(
        &mut self,
        pat: &ir::Pattern<'tcx>,
        qpath: &ir::QPath<'tcx>,
        fields_pats: &[ir::FieldPat<'tcx>],
        ty: Ty<'tcx>,
    ) -> Ty<'tcx> {
        let (variant, struct_ty) = if let Some(ret) = self.check_struct_path(pat, qpath) {
            ret
        } else {
            return self.mk_ty_err();
        };

        self.equate(pat.span, ty, struct_ty);

        let (_adt_ty, substs) = struct_ty.expect_adt();

        // keep track of fields seen to avoid duplicate bindings
        // however, we allow incomplete bindings like javascript
        let mut seen = FxHashMap::default();
        let variant_fields_idents: BiMap<usize, Ident> =
            variant.fields.iter().enumerate().map(|(i, field)| (i, field.ident)).collect();

        for field in fields_pats {
            let field_ty = if !variant_fields_idents.contains_right(&field.ident) {
                self.emit_ty_err(field.span, TypeError::UnknownField(struct_ty, field.ident))
            } else {
                if let Some(span) = seen.insert(field.ident, field.span) {
                    self.emit_ty_err(
                        vec![span, field.span],
                        TypeError::Msg(format!(
                            "field `{}` bound more than once in struct pattern",
                            field.ident,
                        )),
                    );
                }
                let field_idx = variant_fields_idents.get_by_right(&field.ident).copied().unwrap();
                self.record_field_index(field.pat.id, field_idx);
                variant.fields[field_idx].ty(self.tcx, substs)
            };
            self.record_ty(field.pat.id, field_ty);
            self.check_pat(field.pat, field_ty);
        }

        struct_ty
    }

    fn check_pat_box(
        &mut self,
        pat: &ir::Pattern<'tcx>,
        inner: &ir::Pattern<'tcx>,
        ty: Ty<'tcx>,
    ) -> Ty<'tcx> {
        let deref_ty = self.deref_ty(pat.span, ty);
        self.check_pat(inner, deref_ty);
        ty
    }

    fn check_pat_path(
        &mut self,
        pat: &ir::Pattern<'tcx>,
        qpath: &ir::QPath<'tcx>,
        ty: Ty<'tcx>,
    ) -> Ty<'tcx> {
        // before we use `check_expr_path` there are some cases we must handle
        // for example:
        // `Some` has type T -> Option<T>
        // however, we don't want the pattern `Some` to have that same type
        // a valid use of the pattern would be `Some(x)`
        // this should be an error instead as it should be handled under
        // PatKind::Variant not PatKind::Path
        let res = self.resolve_qpath(qpath);
        match res {
            // this is the good case
            Res::Def(_, DefKind::Ctor(CtorKind::Unit)) => (),
            Res::Def(_, DefKind::Ctor(CtorKind::Tuple | CtorKind::Struct)) =>
                return self.emit_ty_err(pat.span, TypeError::UnexpectedVariant(res)),
            Res::Err => return self.set_ty_err(),
            res => unreachable!("unexpected res `{}`", res),
        };
        let path_ty = self.check_qpath(pat, qpath);
        self.equate(pat.span, ty, path_ty);
        path_ty
    }

    fn check_pat_variant(
        &mut self,
        pat: &ir::Pattern<'tcx>,
        qpath: &ir::QPath<'tcx>,
        pats: &[ir::Pattern<'tcx>],
        pat_ty: Ty<'tcx>,
    ) -> Ty<'tcx> {
        let ctor_ty = self.check_qpath(pat, qpath);
        let params = self.tcx.mk_substs(pats.iter().map(|pat| self.new_infer_var(pat.span)));
        for (pat, ty) in pats.iter().zip(params) {
            self.check_pat(pat, ty);
        }
        let fn_ty = self.tcx.mk_fn_ptr(FnSig { params, ret: pat_ty });
        // TODO maybe expected and actual should be the other way around?
        self.equate(pat.span, ctor_ty, fn_ty);
        pat_ty
    }

    fn check_pat_lit(&mut self, expr: &ir::Expr<'tcx>, expected: Ty<'tcx>) -> Ty<'tcx> {
        let lit_ty = self.check_expr(expr);
        self.equate(expr.span, expected, lit_ty);
        lit_ty
    }

    fn check_pat_tuple(
        &mut self,
        pat: &ir::Pattern<'tcx>,
        pats: &[ir::Pattern<'tcx>],
        ty: Ty<'tcx>,
    ) -> Ty<'tcx> {
        // create inference variables for each element
        let tys = self.tcx.mk_substs(pats.iter().map(|pat| self.new_infer_var(pat.span)));
        for (pat, ty) in pats.iter().zip(tys) {
            self.check_pat(pat, ty);
        }
        let pat_ty = self.tcx.mk_tup(tys);
        // we expect `ty` to be a tuple
        self.equate(pat.span, ty, pat_ty);
        pat_ty
    }
}

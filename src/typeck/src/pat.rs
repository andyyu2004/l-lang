use super::FnCtx;
use ir::{self, CtorKind, DefKind};
use lcore::ty::*;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// typechecks a given pattern with its expected type
    pub fn check_pat(&mut self, pat: &ir::Pattern, ty: Ty<'tcx>) -> Ty<'tcx> {
        let pat_ty = match &pat.kind {
            ir::PatternKind::Box(inner) => self.check_pat_box(pat, inner, ty),
            ir::PatternKind::Binding(_, _, mtbl) => self.def_local(pat.id, ty, *mtbl),
            ir::PatternKind::Tuple(pats) => self.check_pat_tuple(pat, pats, ty),
            ir::PatternKind::Lit(expr) => self.check_pat_lit(expr, ty),
            ir::PatternKind::Variant(path, pats) => self.check_pat_variant(pat, path, pats, ty),
            ir::PatternKind::Path(path) => self.check_pat_path(pat, path, ty),
            ir::PatternKind::Wildcard => ty,
        };
        self.write_ty(pat.id, pat_ty)
    }

    fn check_pat_box(&mut self, pat: &ir::Pattern, inner: &ir::Pattern, ty: Ty<'tcx>) -> Ty<'tcx> {
        let deref_ty = self.deref_ty(pat.span, ty);
        self.check_pat(inner, deref_ty);
        ty
    }

    fn check_pat_path(&mut self, pat: &ir::Pattern, path: &ir::Path, ty: Ty<'tcx>) -> Ty<'tcx> {
        // before we use `check_expr_path` there are some cases we must handle
        // for example:
        // `Some` has type T -> Option<T>
        // however, we don't want the pattern `Some` to have that same type
        // a valid use of the pattern would be `Some(x)`
        // this should be an error instead as it should be handled under
        // PatKind::Variant not PatKind::Path
        match path.res {
            // this is the good case
            ir::Res::Def(_, DefKind::Ctor(CtorKind::Unit)) => (),
            ir::Res::Def(_, DefKind::Ctor(CtorKind::Tuple | CtorKind::Struct)) =>
                return self.emit_ty_err(pat.span, TypeError::UnexpectedVariant(path.res)),
            ir::Res::Err => return self.set_ty_err(),
            _ => unreachable!(),
        };
        let path_ty = self.check_expr_path(path);
        self.equate(pat.span, ty, path_ty);
        path_ty
    }

    fn check_pat_variant(
        &mut self,
        pat: &ir::Pattern,
        path: &ir::Path,
        pats: &[ir::Pattern],
        pat_ty: Ty<'tcx>,
    ) -> Ty<'tcx> {
        let ctor_ty = self.check_expr_path(path);
        let params = self.tcx.mk_substs(pats.iter().map(|pat| self.new_infer_var(pat.span)));
        for (pat, ty) in pats.iter().zip(params) {
            self.check_pat(pat, ty);
        }
        let fn_ty = self.tcx.mk_fn_ty(params, pat_ty);
        // TODO maybe expected and actual should be the other way around?
        self.equate(pat.span, ctor_ty, fn_ty);
        pat_ty
    }

    fn check_pat_lit(&mut self, expr: &ir::Expr, expected: Ty<'tcx>) -> Ty<'tcx> {
        let lit_ty = self.check_expr(expr);
        self.equate(expr.span, expected, lit_ty);
        lit_ty
    }

    fn check_pat_tuple(
        &mut self,
        pat: &ir::Pattern,
        pats: &[ir::Pattern],
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

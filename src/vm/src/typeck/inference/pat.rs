use super::FnCtx;
use crate::error::TypeResult;
use crate::ty::*;
use crate::typeck::TyCtx;
use crate::{ast, ir, tir};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// typechecks a given pattern with its expected type
    pub fn check_pat(&mut self, pat: &ir::Pattern, ty: Ty<'tcx>) -> Ty<'tcx> {
        let pat_ty = match &pat.kind {
            ir::PatternKind::Wildcard => ty,
            ir::PatternKind::Binding(ident, _, mtbl) => self.def_local(pat.id, ty, *mtbl),
            ir::PatternKind::Tuple(pats) => self.check_pat_tuple(pat, pats, ty),
            ir::PatternKind::Lit(expr) => self.check_pat_lit(expr, ty),
            ir::PatternKind::Variant(path, pats) => self.check_pat_variant(pat, path, pats, ty),
            ir::PatternKind::Path(path) => self.check_expr_path(path),
        };
        self.write_ty(pat.id, pat_ty)
    }

    fn check_pat_variant(
        &mut self,
        pat: &ir::Pattern,
        path: &ir::Path,
        pats: &[ir::Pattern],
        ty: Ty<'tcx>,
    ) -> Ty<'tcx> {
        // expect path to be either a tuple struct or an enum variant
        // thus we typecheck it as a function
        let (args, ret) = self.check_expr_path(path).expect_fn();
        self.check_pat_tuple(pat, pats, self.mk_tup(args));
        ty
    }

    fn check_pat_lit(&mut self, expr: &ir::Expr, expected: Ty<'tcx>) -> Ty<'tcx> {
        let lit_ty = self.check_expr(expr);
        self.unify(expr.span, expected, lit_ty);
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
        self.unify(pat.span, ty, pat_ty);
        pat_ty
    }
}

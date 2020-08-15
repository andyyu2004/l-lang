use super::FnCtx;
use crate::error::TypeResult;
use crate::ty::*;
use crate::typeck::{TyCtx, TypeckTables};
use crate::{ast, ir, tir};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// typechecks a given pattern with its expected type
    pub fn check_pat(&mut self, pat: &ir::Pattern, ty: Ty<'tcx>) -> Ty<'tcx> {
        // note that the type is recorded for each identifier as well as the whole pattern
        let pat_ty = match &pat.kind {
            ir::PatternKind::Wildcard => ty,
            ir::PatternKind::Binding(ident, _, mtbl) => self.def_local(pat.id, ty, *mtbl),
            ir::PatternKind::Tuple(pats) => self.check_pat_tuple(pats),
            ir::PatternKind::Lit(expr) => self.check_pat_lit(expr, ty),
        };
        self.write_ty(pat.id, pat_ty)
    }

    fn check_pat_lit(&mut self, expr: &ir::Expr, expected: Ty<'tcx>) -> Ty<'tcx> {
        let lit_ty = self.check_expr(expr);
        self.unify(expr.span, expected, lit_ty);
        lit_ty
    }

    fn check_pat_tuple(&mut self, pats: &[ir::Pattern]) -> Ty<'tcx> {
        // create inference variables for each element
        let n = pats.len();
        let tys = self.tcx.mk_substs((0..n).map(|_| self.new_infer_var()));
        for (pat, ty) in pats.iter().zip(tys) {
            self.check_pat(pat, ty);
        }
        let pat_ty = self.tcx.mk_ty(TyKind::Tuple(tys));
        pat_ty
    }
}

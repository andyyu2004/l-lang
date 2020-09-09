use super::FnCtx;
use crate::error::{TypeError, TypeResult};
use crate::ty::*;
use crate::typeck::TyCtx;
use crate::{ast, ir, tir};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// typechecks a given pattern with its expected type
    pub fn check_pat(&mut self, pat: &ir::Pattern, ty: Ty<'tcx>) -> Ty<'tcx> {
        // note that the type is recorded for each identifier as well as the whole pattern
        let pat_ty = match (&pat.kind, &ty.kind) {
            (ir::PatternKind::Wildcard, _) => ty,
            (ir::PatternKind::Binding(ident, _, mtbl), _) => self.def_local(pat.id, ty, *mtbl),
            (ir::PatternKind::Lit(expr), _) => self.check_pat_lit(expr, ty),
            (ir::PatternKind::Tuple(pats), TyKind::Tuple(tys)) if pats.len() == tys.len() => {
                for (pat, ty) in pats.iter().zip(tys.iter()) {
                    self.check_pat(pat, ty);
                }
                ty
            }
            (ir::PatternKind::Tuple(pats), TyKind::Tuple(tys)) =>
                self.emit_ty_err(pat.span, TypeError::TupleSizeMismatch(pats.len(), tys.len())),

            _ => self.emit_ty_err(
                pat.span,
                TypeError::Msg(format!("failed to match pattern against expect type `{}`", ty)),
            ),
        };
        self.write_ty(pat.id, pat_ty)
    }

    fn check_pat_lit(&mut self, expr: &ir::Expr, expected: Ty<'tcx>) -> Ty<'tcx> {
        let lit_ty = self.check_expr(expr);
        self.unify(expr.span, expected, lit_ty);
        lit_ty
    }
}

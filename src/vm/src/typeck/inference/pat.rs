use super::FnCtx;
use crate::error::{TypeError, TypeResult};
use crate::ir::{self, CtorKind, DefKind};
use crate::ty::*;
use crate::typeck::TyCtx;
use crate::{ast, tir};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// typechecks a given pattern with its expected type
    pub fn check_pat(&mut self, pat: &ir::Pattern, ty: Ty<'tcx>) -> Ty<'tcx> {
        let pat_ty = match &pat.kind {
            ir::PatternKind::Wildcard => ty,
            ir::PatternKind::Binding(ident, _, mtbl) => self.def_local(pat.id, ty, *mtbl),
            ir::PatternKind::Tuple(pats) => self.check_pat_tuple(pat, pats, ty),
            ir::PatternKind::Lit(expr) => self.check_pat_lit(expr, ty),
            ir::PatternKind::Variant(path, pats) => self.check_pat_variant(pat, path, pats, ty),
            ir::PatternKind::Path(path) => self.check_pat_path(pat, path),
        };
        self.write_ty(pat.id, pat_ty)
    }

    fn check_pat_path(&mut self, pat: &ir::Pattern, path: &ir::Path) -> Ty<'tcx> {
        // before we use `check_expr_path` there are some cases we must handle
        // for example
        // Some has type T -> Option<T>
        // however, we don't want the pattern `Some` to have that same type
        // indeed, this should be an error instead
        match path.res {
            // this is the good case
            ir::Res::Def(_, DefKind::Ctor(CtorKind::Unit)) => (),
            ir::Res::Def(_, DefKind::Ctor(CtorKind::Tuple | CtorKind::Struct)) =>
                return self.emit_ty_err(pat.span, TypeError::UnexpectedVariant(path.res)),
            ir::Res::Err => return self.set_ty_err(),
            _ => unreachable!(),
        };
        self.check_expr_path(path)
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
        self.unify(pat.span, ctor_ty, fn_ty);
        pat_ty
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

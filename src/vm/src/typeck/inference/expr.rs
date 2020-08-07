use super::FnCtx;
use crate::error::TypeResult;
use crate::ir::{DefId, DefKind};
use crate::ty::*;
use crate::typeck::{TyCtx, TypeckTables};
use crate::{ast, ir, tir};
use itertools::Itertools;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn check_expr(&mut self, expr: &ir::Expr) -> Ty<'tcx> {
        let ty = match &expr.kind {
            ir::ExprKind::Lit(lit) => self.check_lit(lit),
            ir::ExprKind::Bin(op, l, r) => self.check_binop(*op, l, r),
            ir::ExprKind::Unary(_, _) => todo!(),
            ir::ExprKind::Block(block) => self.check_block(block),
            ir::ExprKind::Path(path) => self.check_expr_path(path),
            ir::ExprKind::Tuple(xs) => self.check_expr_tuple(xs),
            ir::ExprKind::Lambda(sig, body) => self.check_lambda_expr(sig, body),
            ir::ExprKind::Call(f, args) => self.check_call_expr(expr, f, args),
            ir::ExprKind::Match(expr, arms, src) => self.check_match_expr(expr, arms, src),
        };
        self.write_ty(expr.id, ty)
    }

    fn check_match_expr(
        &mut self,
        expr: &ir::Expr,
        arms: &[ir::Arm],
        src: &ir::MatchSource,
    ) -> Ty<'tcx> {
        let expr_ty = self.check_expr(expr);
        match src {
            ir::MatchSource::If => self.unify(expr.span, self.tcx.types.boolean, expr_ty),
            ir::MatchSource::Match => {}
        };

        // check that each arm pattern is the same type as the scrutinee
        for arm in arms {
            self.check_pat(arm.pat, expr_ty);
        }

        // special case when match has no arms
        if arms.is_empty() {
            return self.tcx.types.unit;
        }

        // otherwise, consider the last arm's body to be the expected type
        let n = arms.len() - 1;
        let expected_ty = self.check_expr(arms[n].body);
        arms[..n].iter().for_each(|arm| {
            let arm_ty = self.check_expr(arm.body);
            arm.guard.map(|expr| {
                let guard_ty = self.check_expr(expr);
                self.unify(expr.span, self.tcx.types.boolean, guard_ty);
            });
            self.unify(arm.span, expected_ty, arm_ty);
        });
        expected_ty
    }

    fn check_call_expr(&mut self, expr: &ir::Expr, f: &ir::Expr, args: &[ir::Expr]) -> Ty<'tcx> {
        let ret_ty = self.new_infer_var();
        let f_ty = self.check_expr(f);
        let arg_tys = self.check_expr_list(args);
        let ty = self.tcx.mk_ty(TyKind::Fn(arg_tys, ret_ty));
        self.unify(expr.span, f_ty, ty);
        ret_ty
    }

    fn check_lambda_expr(&mut self, sig: &ir::FnSig, body: &ir::Body) -> Ty<'tcx> {
        self.check_fn(sig, body).1
    }

    /// inputs are the types from the type signature (or inference variables)
    /// adds the parameters to locals and typechecks the expr of the body
    pub fn check_body(&mut self, fn_ty: Ty<'tcx>, body: &ir::Body) -> Ty<'tcx> {
        let (param_tys, ret_ty) = fn_ty.expect_fn();
        debug_assert_eq!(param_tys.len(), body.params.len());
        for (param, ty) in body.params.iter().zip(param_tys) {
            self.check_pat(param.pat, ty);
        }
        let body_ty = self.check_expr(body.expr);
        self.unify(body.expr.span, self.expected_ret_ty, ret_ty);
        self.unify(body.expr.span, ret_ty, body_ty);
        body_ty
    }

    fn check_expr_list(&mut self, xs: &[ir::Expr]) -> SubstRef<'tcx> {
        let tcx = self.tcx;
        let tys = xs.iter().map(|expr| self.check_expr(expr));
        tcx.mk_substs(tys)
    }

    fn check_expr_tuple(&mut self, xs: &[ir::Expr]) -> Ty<'tcx> {
        let tcx = self.tcx;
        let tys = xs.iter().map(|expr| self.check_expr(expr));
        tcx.mk_tup(tys)
    }

    fn check_expr_path(&mut self, path: &ir::Path) -> Ty<'tcx> {
        match path.res {
            ir::Res::Local(id) => self.local_ty(id),
            ir::Res::Def(def_id, def_kind) => self.check_expr_path_def(def_id, def_kind),
            ir::Res::PrimTy(_) => panic!("found type resolution in value namespace"),
        }
    }

    fn check_expr_path_def(&mut self, def_id: DefId, def_kind: DefKind) -> Ty<'tcx> {
        match def_kind {
            // instantiate ty params
            DefKind::Fn => self.instantiate(self.tcx.item_ty(def_id)),
            DefKind::Enum => todo!(),
            DefKind::TyParam(_) => panic!(),
        }
    }

    fn check_block(&mut self, block: &ir::Block) -> Ty<'tcx> {
        block.stmts.iter().for_each(|stmt| self.check_stmt(stmt));
        match &block.expr {
            Some(expr) => self.check_expr(expr),
            None => {
                // explicitly handle the case when the final stmt is a return stmt
                match block.stmts.last().map(|stmt| &stmt.kind) {
                    Some(ir::StmtKind::Ret(_)) => self.tcx.types.never,
                    _ => self.tcx.types.unit,
                }
            }
        }
    }

    fn check_binop(&mut self, op: ast::BinOp, l: &ir::Expr, r: &ir::Expr) -> Ty<'tcx> {
        let tl = self.check_expr(l);
        let tr = self.check_expr(r);
        match op {
            // only allow these operations on numbers for now
            ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => {
                self.unify(l.span, self.tcx.types.num, tl);
                self.unify(r.span, tl, tr);
                tl
            }
            ast::BinOp::Lt | ast::BinOp::Gt => {
                self.unify(l.span, self.tcx.types.num, tl);
                self.unify(r.span, tl, tr);
                self.tcx.types.boolean
            }
        }
    }

    fn check_lit(&self, lit: &ast::Lit) -> Ty<'tcx> {
        match lit {
            ast::Lit::Num(_) => self.tcx.types.num,
            ast::Lit::Bool(_) => self.tcx.types.boolean,
        }
    }
}

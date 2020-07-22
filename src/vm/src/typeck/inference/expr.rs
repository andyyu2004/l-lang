use super::FnCtx;
use crate::error::TypeResult;
use crate::ir::{DefId, DefKind};
use crate::ty::*;
use crate::typeck::{TyCtx, TypeckTables};
use crate::{ast, ir, tir};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn check_expr(&mut self, expr: &ir::Expr) -> Ty<'tcx> {
        let ty = match &expr.kind {
            ir::ExprKind::Lit(lit) => self.check_lit(lit),
            ir::ExprKind::Bin(op, l, r) => self.check_binop(*op, l, r),
            ir::ExprKind::Unary(_, _) => todo!(),
            ir::ExprKind::Block(block) => self.check_block(block),
            ir::ExprKind::Path(path) => self.check_expr_path(path),
            ir::ExprKind::Tuple(xs) => self.check_expr_tuple(xs),
            ir::ExprKind::Lambda(sig, body) => self.check_lambda_expr(expr, sig, body),
            ir::ExprKind::Call(f, args) => self.check_call_expr(expr, f, args),
        };
        self.write_ty(expr.id, ty)
    }

    fn check_call_expr(&mut self, expr: &ir::Expr, f: &ir::Expr, args: &[ir::Expr]) -> Ty<'tcx> {
        let ret_ty = self.new_infer_var();
        let f_ty = self.check_expr(f);
        let arg_tys = self.check_expr_list(args);
        let ty = self.tcx.mk_ty(TyKind::Fn(arg_tys, ret_ty));
        self.unify(expr.span, f_ty, ty);
        ret_ty
    }

    fn check_lambda_expr(&mut self, expr: &ir::Expr, sig: &ir::FnSig, body: &ir::Body) -> Ty<'tcx> {
        let param_tys = self.tcx.mk_substs(sig.inputs.iter().map(|ty| self.lower_ty(ty)));
        let ret_ty = sig.output.map(|ty| self.lower_ty(ty)).unwrap_or(self.new_infer_var());
        let body_ty = self.check_body(param_tys, body);
        self.unify(expr.span, ret_ty, body_ty);
        self.tcx.mk_ty(TyKind::Fn(param_tys, ret_ty))
    }

    /// inputs are the input types from the type signature (or inference variables)
    /// adds the parameters to locals and typechecks the expr of the body
    pub fn check_body(&mut self, param_tys: SubstRef<'tcx>, body: &ir::Body) -> Ty<'tcx> {
        debug_assert_eq!(param_tys.len(), body.params.len());
        for (param, ty) in body.params.iter().zip(param_tys) {
            self.check_pat(param.pat, ty);
        }
        self.check_expr(body.expr)
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
            DefKind::Fn => self.tcx.item_ty(def_id),
        }
    }

    fn check_block(&mut self, block: &ir::Block) -> Ty<'tcx> {
        block.stmts.iter().for_each(|stmt| self.check_stmt(stmt));
        match &block.expr {
            Some(expr) => self.check_expr(expr),
            None => self.tcx.types.unit,
        }
    }

    fn check_binop(&mut self, op: ast::BinOp, l: &ir::Expr, r: &ir::Expr) -> Ty<'tcx> {
        let tl = self.check_expr(l);
        let tr = self.check_expr(r);
        match op {
            ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => {
                self.unify(r.span, tl, tr);
                tl
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

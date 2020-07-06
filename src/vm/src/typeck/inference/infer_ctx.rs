use super::*;
use crate::ty::*;
use crate::{
    ast, ir, tir, typeck::{InferResult, TyCtx}
};
use ena::unify as ut;
use std::marker::PhantomData;
use std::{cell::RefCell, ops::Deref};

crate struct InferCtxBuilder<'tcx> {
    tcx: TyCtx<'tcx>,
    inference_ctx: InferCtxInner<'tcx>,
}

impl<'tcx> InferCtxBuilder<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>) -> Self {
        Self { tcx, inference_ctx: Default::default() }
    }
}

impl<'tcx> InferCtxBuilder<'tcx> {
    crate fn enter<R>(self, f: impl for<'a> FnOnce(InferCtx<'a, 'tcx>) -> R) -> R {
        let infer_ctx = InferCtx {
            tcx: self.tcx,
            inner: RefCell::new(self.inference_ctx),
            marker: &PhantomData,
        };
        f(infer_ctx)
    }
}

#[derive(Default)]
crate struct InferCtxInner<'tcx> {
    type_variable_storage: TypeVariableStorage<'tcx>,
    undo_log: InferCtxUndoLogs<'tcx>,
}

impl<'tcx> InferCtxInner<'tcx> {
    pub fn type_variables(&mut self) -> TypeVariableTable<'_, 'tcx> {
        self.type_variable_storage.with_log(&mut self.undo_log)
    }
}

crate struct InferCtx<'a, 'tcx> {
    crate tcx: TyCtx<'tcx>,
    crate inner: RefCell<InferCtxInner<'tcx>>,
    marker: &'a PhantomData<()>,
}

impl<'a, 'tcx> InferCtx<'a, 'tcx> {
    pub fn gen_substs(&self) -> InferResult<'tcx, SubstRef<'tcx>> {
        let substs = self.inner.borrow_mut().type_variables().gen_substs()?;
        Ok(self.tcx.intern_substs(&substs))
    }

    pub fn infer_expr(&'a self, expr: &ir::Expr<'_>) -> InferResult<'tcx, &'tcx tir::Expr<'tcx>> {
        let tir_expr = self.type_expr(expr)?;
        let substs = self.gen_substs()?;
        Ok(tir_expr.subst(self.tcx, substs))
    }

    fn type_expr(&'a self, expr: &ir::Expr<'_>) -> InferResult<'tcx, &'tcx tir::Expr<'tcx>> {
        Ok(self.tcx.alloc_tir(self.type_expr_inner(expr)?))
    }

    fn type_expr_inner(&'a self, expr: &ir::Expr<'_>) -> InferResult<'tcx, tir::Expr<'tcx>> {
        match expr.kind {
            ir::ExprKind::Lit(lit) => Ok(self.type_expr_lit(expr, lit)),
            ir::ExprKind::Bin(op, lhs, rhs) => self.type_expr_binary(expr, op, lhs, rhs),
            ir::ExprKind::Unary(op, expr) => todo!(),
        }
    }

    pub fn type_expr_binary(
        &'a self,
        expr: &ir::Expr<'_>,
        op: ast::BinOp,
        lhs: &ir::Expr<'_>,
        rhs: &ir::Expr<'_>,
    ) -> InferResult<'tcx, tir::Expr<'tcx>> {
        let l = self.type_expr(lhs)?;
        let r = self.type_expr(rhs)?;
        let kind = tir::ExprKind::Bin(op, l, r);
        let ty = match op {
            ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => {
                let new_var = self.inner.borrow_mut().type_variables().new_ty_var();
                let infer_var = self.tcx.mk_ty(TyKind::Infer(InferTy::TyVar(new_var)));
                self.at(expr.span).ceq(infer_var, &self.tcx.types.num)?;
                self.at(lhs.span).ceq(&l.ty, infer_var)?;
                self.at(rhs.span).ceq(&r.ty, infer_var)?;
                infer_var
            }
        };

        Ok(tir::Expr { ty, span: expr.span, kind })
    }

    pub fn type_expr_lit(&self, expr: &ir::Expr<'_>, lit: ast::Lit) -> tir::Expr<'tcx> {
        let ty = match lit {
            ast::Lit::Num(_) => self.tcx.types.num,
            ast::Lit::Bool(_) => self.tcx.types.boolean,
        };
        tir::Expr { ty, span: expr.span, kind: tir::ExprKind::Lit(lit) }
    }
}

use super::{Constraint, Constraints};
use crate::{ast, ir, tir, typeck::TyCtx};
use std::{cell::RefCell, ops::Deref};

crate struct InferCtxBuilder<'tcx> {
    tcx: TyCtx<'tcx>,
    inference_ctx: InferCtxInner<'tcx>,
}

impl<'tcx> InferCtxBuilder<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>) -> Self {
        Self {
            tcx,
            inference_ctx: Default::default(),
        }
    }
}

#[derive(Default)]
crate struct InferCtxInner<'tcx> {
    constraints: RefCell<Constraints<'tcx>>,
}

impl<'tcx> InferCtxInner<'tcx> {
    pub fn constrain(&self, constraint: Constraint<'tcx>) {
        self.constraints.borrow_mut().push(constraint)
    }
}

impl<'tcx> InferCtxBuilder<'tcx> {
    crate fn enter<R>(self, f: impl for<'a> FnOnce(InferCtx<'a, 'tcx>) -> R) -> R {
        let infer_ctx = InferCtx {
            tcx: self.tcx,
            inner: &self.inference_ctx,
        };
        f(infer_ctx)
    }
}

crate struct InferCtx<'a, 'tcx> {
    crate tcx: TyCtx<'tcx>,
    inner: &'a InferCtxInner<'tcx>,
}

impl<'a, 'tcx> Deref for InferCtx<'a, 'tcx> {
    type Target = InferCtxInner<'tcx>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, 'tcx> InferCtx<'a, 'tcx> {
    pub fn infer_expr(&self, expr: &ir::Expr<'_>) -> &'tcx tir::Expr<'tcx> {
        let expr = self.type_expr(expr);
        println!("constraints: {}", self.constraints.borrow());
        let constraints = self.constraints.borrow();
        let subst = self.solve(&constraints);
        expr
    }

    fn type_expr(&'a self, expr: &ir::Expr<'_>) -> &'tcx tir::Expr<'tcx> {
        self.tcx.alloc_tir(self.type_expr_inner(expr))
    }

    fn type_expr_inner(&'a self, expr: &ir::Expr<'_>) -> tir::Expr<'tcx> {
        match expr.kind {
            ir::ExprKind::Lit(lit) => self.type_expr_lit(expr, lit),
            ir::ExprKind::Bin(op, lhs, rhs) => self.type_expr_binary(expr, op, lhs, rhs),
            ir::ExprKind::Unary(op, expr) => todo!(),
        }
    }

    pub fn type_expr_binary(
        &self,
        expr: &ir::Expr<'_>,
        op: ast::BinOp,
        lhs: &ir::Expr<'_>,
        rhs: &ir::Expr<'_>,
    ) -> tir::Expr<'tcx> {
        let l = self.type_expr(lhs);
        let r = self.type_expr(rhs);
        let kind = tir::ExprKind::Bin(op, l, r);
        let ty = match op {
            ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => {
                self.at(expr.span).ceq(&l.ty, &r.ty);
                self.at(lhs.span).ceq(&l.ty, self.tcx.types.num);
                self.at(rhs.span).ceq(&r.ty, self.tcx.types.num);
                &l.ty
            }
        };

        tir::Expr {
            ty,
            span: expr.span,
            kind,
        }
    }

    pub fn type_expr_lit(&self, expr: &ir::Expr<'_>, lit: ast::Lit) -> tir::Expr<'tcx> {
        let ty = match lit {
            ast::Lit::Num(_) => self.tcx.types.num,
            ast::Lit::Bool(_) => self.tcx.types.boolean,
        };
        tir::Expr {
            ty,
            span: expr.span,
            kind: tir::ExprKind::Lit(lit),
        }
    }
}

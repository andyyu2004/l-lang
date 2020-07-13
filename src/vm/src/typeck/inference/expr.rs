use super::FnCtx;
use crate::error::TypeResult;
use crate::ty::*;
use crate::typeck::{TyCtx, TypeckTables};
use crate::{ast, ir, tir};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn check_expr(&self, expr: &ir::Expr) -> Ty<'tcx> {
        let ty = match &expr.kind {
            ir::ExprKind::Lit(lit) => self.check_lit(lit),
            ir::ExprKind::Bin(op, l, r) => self.check_binop(*op, l, r),
            ir::ExprKind::Unary(_, _) => todo!(),
            ir::ExprKind::Block(block) => self.check_block(block),
        };
        self.write_ty(expr.id, ty);
        ty
    }

    pub fn check_binop(&self, op: ast::BinOp, l: &ir::Expr, r: &ir::Expr) -> Ty<'tcx> {
        let tl = self.check_expr(l);
        let tr = self.check_expr(r);
        match op {
            ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => {
                self.expect_eq(r.span, tl, tr);
                tl
            }
        }
    }

    pub fn check_lit(&self, lit: &ast::Lit) -> Ty<'tcx> {
        match lit {
            ast::Lit::Num(_) => self.tcx.types.num,
            ast::Lit::Bool(_) => self.tcx.types.boolean,
        }
    }
}

// impl<'a, 'tcx> FnCtx<'a, 'tcx> {
//     pub fn gen_substs(&self) -> TypeResult<'tcx, SubstRef<'tcx>> {
//         let substs = self.inner.borrow_mut().type_variables().gen_substs()?;
//         Ok(self.tcx.intern_substs(&substs))
//     }

//     pub fn infer_expr(&'a self, expr: &ir::Expr<'_>) -> TypeResult<'tcx, &'tcx tir::Expr<'tcx>> {
//         let tir_expr = self.type_expr(expr)?;
//         let substs = self.gen_substs()?;
//         Ok(tir_expr.subst(self.tcx, substs))
//     }

//     fn type_expr(&'a self, expr: &ir::Expr<'_>) -> TypeResult<'tcx, &'tcx tir::Expr<'tcx>> {
//         Ok(self.tcx.alloc_tir(self.type_expr_inner(expr)?))
//     }

//     fn type_expr_inner(&'a self, expr: &ir::Expr<'_>) -> TypeResult<'tcx, tir::Expr<'tcx>> {
//         match expr.kind {
//             ir::ExprKind::Lit(lit) => Ok(self.type_expr_lit(expr, lit)),
//             ir::ExprKind::Bin(op, lhs, rhs) => self.type_expr_binary(expr, op, lhs, rhs),
//             ir::ExprKind::Unary(op, expr) => todo!(),
//             ir::ExprKind::Block(block) => todo!(),
//         }
//     }

//     pub fn type_expr_binary(
//         &'a self,
//         expr: &ir::Expr<'_>,
//         op: ast::BinOp,
//         lhs: &ir::Expr<'_>,
//         rhs: &ir::Expr<'_>,
//     ) -> TypeResult<'tcx, tir::Expr<'tcx>> {
//         let l = self.type_expr(lhs)?;
//         let r = self.type_expr(rhs)?;
//         let kind = tir::ExprKind::Bin(op, l, r);
//         let ty = match op {
//             ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => {
//                 let new_var = self.inner.borrow_mut().type_variables().new_ty_var();
//                 let infer_var = self.tcx.mk_ty(TyKind::Infer(InferTy::TyVar(new_var)));
//                 self.at(expr.span).equate(infer_var, &self.tcx.types.num)?;
//                 self.at(lhs.span).equate(l.ty, infer_var)?;
//                 self.at(rhs.span).equate(r.ty, infer_var)?;
//                 infer_var
//             }
//         };

//         Ok(tir::Expr { ty, span: expr.span, kind })
//     }

//     pub fn type_expr_lit(&self, expr: &ir::Expr<'_>, lit: ast::Lit) -> tir::Expr<'tcx> {
//         let ty = match lit {
//             ast::Lit::Num(_) => self.tcx.types.num,
//             ast::Lit::Bool(_) => self.tcx.types.boolean,
//         };
//         tir::Expr { ty, span: expr.span, kind: tir::ExprKind::Lit(lit) }
//     }
// }

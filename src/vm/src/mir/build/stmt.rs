use super::{BlockAnd, BlockAndExt, Builder};
use crate::mir::{BlockId, Lvalue, Rvalue, VarKind};
use crate::set;
use crate::tir;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn build_stmt(&mut self, mut block: BlockId, stmt: &tir::Stmt<'tcx>) -> BlockAnd<()> {
        let info = self.span_info(stmt.span);
        match stmt.kind {
            tir::StmtKind::Let(tir::Let { id, pat, init }) => {
                let lvalue = set!(block = self.declare_pat(block, pat)).into();
                match init {
                    Some(expr) => self.write_expr(block, lvalue, expr),
                    None => todo!(),
                }
            }
            tir::StmtKind::Expr(expr) => self.build_expr_stmt(block, expr),
        }
    }

    crate fn build_expr_stmt(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<()> {
        match expr.kind {
            tir::ExprKind::Ret(_) => todo!(),
            tir::ExprKind::Const(_)
            | tir::ExprKind::Bin(_, _, _)
            | tir::ExprKind::Unary(_, _)
            | tir::ExprKind::Block(_)
            | tir::ExprKind::VarRef(_)
            | tir::ExprKind::ItemRef(_)
            | tir::ExprKind::Tuple(_)
            | tir::ExprKind::Lambda(_)
            | tir::ExprKind::Call(_, _)
            | tir::ExprKind::Match(_, _)
            | tir::ExprKind::Assign(_, _) => {
                // write the expr stmt into some (unused) tmp var
                set!(block = self.as_tmp(block, expr));
                block.unit()
            }
        }
    }
}

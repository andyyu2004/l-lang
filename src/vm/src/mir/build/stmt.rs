use super::{BlockAnd, BlockAndExt, Builder};
use crate::mir::{BlockId, Lvalue, Rvalue, TerminatorKind, VarKind};
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

    // does not allocate any locals as there is no result produced or the result is ignored
    crate fn build_expr_stmt(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        match expr.kind {
            tir::ExprKind::Ret(ret_expr) => {
                let ret_lvalue = self.ret_lvalue();
                match ret_expr {
                    Some(expr) => {
                        set!(block = self.write_expr(block, ret_lvalue, expr));
                    }
                    None => self.cfg.push_assign_unit(info, block, ret_lvalue),
                }
                self.cfg.terminate(info, block, TerminatorKind::Return);
                self.cfg.append_basic_block().unit()
            }
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

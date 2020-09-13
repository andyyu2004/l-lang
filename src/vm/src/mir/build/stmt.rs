use super::{BlockAnd, BlockAndExt, Builder};
use crate::mir::{BlockId, Lvalue, Rvalue, TerminatorKind, VarKind};
use crate::set;
use crate::tir;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn build_stmt(&mut self, mut block: BlockId, stmt: &tir::Stmt<'tcx>) -> BlockAnd<()> {
        let info = self.span_info(stmt.span);
        match stmt.kind {
            tir::StmtKind::Let(tir::Let { id, pat, init }) => match init {
                Some(expr) => {
                    let rvalue = set!(block = self.as_lvalue(block, expr));
                    self.bind_pat_to_lvalue(block, pat, rvalue)
                }
                None => todo!(),
            },
            tir::StmtKind::Expr(expr) => self.build_expr_stmt(block, expr),
        }
    }

    // some expressions can have a more efficient implementation if we know the return value will
    // be unused (as it is an expression statement)
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
                    None => self.push_assign_unit(info, block, ret_lvalue),
                }
                self.terminate(info, block, TerminatorKind::Return);
                self.append_basic_block().unit()
            }
            tir::ExprKind::Assign(l, r) => {
                let lvalue = set!(block = self.as_lvalue(block, l));
                let rvalue = set!(block = self.as_rvalue(block, r));
                self.push_assignment(info, block, lvalue, rvalue);
                block.unit()
            }
            tir::ExprKind::Const(_)
            | tir::ExprKind::Bin(_, _, _)
            | tir::ExprKind::Unary(_, _)
            | tir::ExprKind::Block(_)
            | tir::ExprKind::VarRef(_)
            | tir::ExprKind::ItemRef(_)
            | tir::ExprKind::Tuple(_)
            | tir::ExprKind::Lambda(_)
            | tir::ExprKind::Deref(_)
            | tir::ExprKind::Field(..)
            | tir::ExprKind::Call(_, _)
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Box(..)
            | tir::ExprKind::Match(_, _) => {
                // write the expr stmt into some (unused) tmp var
                set!(block = self.as_tmp(block, expr));
                block.unit()
            }
        }
    }
}

use super::*;

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
    crate fn build_stmt(&mut self, mut block: BlockId, stmt: &tir::Stmt<'tcx>) -> BlockAnd<()> {
        match &stmt.kind {
            tir::StmtKind::Let(tir::Let { pat, init, .. }) => match init {
                Some(expr) => {
                    let lvalue = set!(block = self.as_lvalue(block, expr));
                    self.bind_pat_to_lvalue(block, &pat, lvalue)
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
        match &expr.kind {
            tir::ExprKind::Ret(ret_expr) => {
                let ret_lvalue = self.ret_lvalue();
                match ret_expr {
                    Some(ret_expr) => set!(block = self.write_expr(block, ret_lvalue, ret_expr)),
                    None => self.push_assign_unit(info, block, ret_lvalue),
                }
                self.terminate(info, block, TerminatorKind::Return);
                // TODO exit scope and do ref counting properly
                self.append_basic_block().unit()
            }
            tir::ExprKind::Assign(l, r) => {
                let lvalue = set!(block = self.as_lvalue(block, l));
                let rvalue = set!(block = self.as_rvalue(block, r));
                self.push_assignment(info, block, lvalue, rvalue);
                block.unit()
            }
            tir::ExprKind::Break => self.break_scope(info, block, BreakType::Break),
            tir::ExprKind::Continue => self.break_scope(info, block, BreakType::Continue),
            tir::ExprKind::Box(..)
            | tir::ExprKind::Loop(..)
            | tir::ExprKind::Const(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Unary(..)
            | tir::ExprKind::Block(..)
            | tir::ExprKind::VarRef(..)
            | tir::ExprKind::ItemRef(..)
            | tir::ExprKind::Tuple(..)
            | tir::ExprKind::Ref(..)
            | tir::ExprKind::Deref(..)
            | tir::ExprKind::Field(..)
            | tir::ExprKind::Call(..)
            | tir::ExprKind::Closure { .. }
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Match(..) => {
                debug_assert_ne!(expr.ty, self.tcx.types.never);
                // write the expr stmt into some (unused) tmp var
                set!(block = self.as_tmp(block, expr));
                block.unit()
            }
        }
    }
}

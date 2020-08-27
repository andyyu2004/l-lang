use super::{BlockAnd, BlockAndExt, Builder};
use crate::mir::{BlockId, Lvalue, Rvalue, VarKind};
use crate::set;
use crate::tir;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn build_stmt(&mut self, mut block: BlockId, stmt: &tir::Stmt<'tcx>) -> BlockAnd<()> {
        let info = self.span_info(stmt.span);
        match stmt.kind {
            tir::StmtKind::Let(tir::Let { id, pat, init }) => match pat.kind {
                tir::PatternKind::Wildcard => todo!(),
                tir::PatternKind::Binding(ident, _) => {
                    let var = self.alloc_local(pat);
                    match init {
                        Some(expr) => self.write_expr(block, Lvalue::from(var), expr),
                        None => todo!(),
                    }
                }
                tir::PatternKind::Field(_) => todo!(),
                tir::PatternKind::Lit(_) => unreachable!(),
            },
            tir::StmtKind::Expr(expr) => {
                // write the expr stmt into some unused tmp
                set!(block = self.as_tmp(block, expr));
                block.unit()
            }
        }
    }
}

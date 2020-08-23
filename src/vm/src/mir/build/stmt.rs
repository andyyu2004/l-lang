use super::{BlockAnd, BlockAndExt, Builder};
use crate::mir::{BlockId, Lvalue, Rvalue};
use crate::set;
use crate::tir;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn stmt(&mut self, block: BlockId, stmt: &tir::Stmt<'tcx>) -> BlockAnd<()> {
        match stmt.kind {
            tir::StmtKind::Let(_) => todo!(),
            tir::StmtKind::Expr(_) => todo!(),
        }
    }
}

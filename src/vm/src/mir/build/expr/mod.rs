use super::{BlockAnd, BlockAndExt, Builder};
use crate::mir::{BlockId, Lvalue, Rvalue};
use crate::set;
use crate::tir;

mod operand;
mod rvalue;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    /// compiles `expr` into `lvalue`
    pub fn expr(
        &mut self,
        mut block: BlockId,
        expr: &'tcx tir::Expr<'tcx>,
        lvalue: Lvalue<'tcx>,
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        match expr.kind {
            tir::ExprKind::Block(ir) => self.ir_block(block, ir),
            tir::ExprKind::VarRef(_) => todo!(),
            tir::ExprKind::ItemRef(_) => todo!(),
            tir::ExprKind::Tuple(_) => todo!(),
            tir::ExprKind::Lambda(_) => todo!(),
            tir::ExprKind::Call(_, _) => todo!(),
            tir::ExprKind::Match(_, _) => todo!(),
            tir::ExprKind::Assign(_, _) => todo!(),
            tir::ExprKind::Unary(..) | tir::ExprKind::Const(..) | tir::ExprKind::Bin(..) => {
                let rvalue = set!(block = self.as_rvalue(block, expr));
                self.cfg.push_assignment(info, block, lvalue, rvalue);
                block.unit()
            }
            tir::ExprKind::Ret(_) => todo!(),
        }
    }

    fn ir_block(&mut self, mut block: BlockId, ir: &'tcx tir::Block<'tcx>) -> BlockAnd<()> {
        for stmt in ir.stmts {
            set!(block = self.stmt(block, stmt));
        }
        block.unit()
    }
}

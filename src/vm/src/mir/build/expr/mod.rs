use super::{BlockAnd, BlockAndExt, Builder};
use crate::mir::{BlockId, Lvalue, Rvalue};
use crate::set;
use crate::tir;

mod constant;
mod operand;
mod rvalue;
mod tmp;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    /// writes `expr` into `lvalue`
    pub fn write_expr(
        &mut self,
        mut block: BlockId,
        lvalue: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        match expr.kind {
            tir::ExprKind::Block(ir) => self.ir_block(block, lvalue, expr, ir),
            tir::ExprKind::VarRef(_) => todo!(),
            tir::ExprKind::ItemRef(_) => todo!(),
            tir::ExprKind::Tuple(_) => todo!(),
            tir::ExprKind::Lambda(_) => todo!(),
            tir::ExprKind::Call(_, _) => todo!(),
            tir::ExprKind::Match(_, _) => todo!(),
            tir::ExprKind::Assign(_, _) => todo!(),
            tir::ExprKind::Unary(..) | tir::ExprKind::Bin(..) | tir::ExprKind::Const(..) => {
                let rvalue = set!(block = self.as_rvalue(block, expr));
                self.cfg.push_assignment(info, block, lvalue, rvalue);
                block.unit()
            }
            tir::ExprKind::Ret(_) => todo!(),
        }
    }

    fn ir_block(
        &mut self,
        mut block: BlockId,
        lvalue: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        ir: &tir::Block<'tcx>,
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        for stmt in ir.stmts {
            set!(block = self.build_stmt(block, stmt));
        }

        if let Some(expr) = ir.expr {
            set!(block = self.write_expr(block, lvalue, expr));
        } else {
            self.cfg.push_unit(info, block, lvalue)
        }
        block.unit()
    }
}

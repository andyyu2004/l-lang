use super::{BlockAnd, BlockAndExt, Builder};
use crate::mir::{BlockId, Lvalue, Operand, Rvalue, SpanInfo, TerminatorKind};
use crate::set;
use crate::span::Span;
use crate::tir;
use itertools::Itertools;

mod constant;
mod lvalue;
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
            tir::ExprKind::ItemRef(_) => todo!(),
            tir::ExprKind::Tuple(_) => todo!(),
            tir::ExprKind::Lambda(_) => todo!(),
            tir::ExprKind::Call(f, args) => self.build_call(block, expr, lvalue, f, args),
            tir::ExprKind::Match(scrut, arms) => self.build_match(block, expr, lvalue, scrut, arms),
            tir::ExprKind::Assign(lhs, rhs) => {
                let rvalue = set!(block = self.as_rvalue(block, rhs));
                let lvalue = set!(block = self.as_lvalue(block, lhs));
                self.cfg.push_assignment(info, block, lvalue, rvalue);
                block.unit()
            }
            tir::ExprKind::VarRef(..)
            | tir::ExprKind::Unary(..)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Const(..) => {
                let rvalue = set!(block = self.as_rvalue(block, expr));
                self.cfg.push_assignment(info, block, lvalue, rvalue);
                block.unit()
            }
            tir::ExprKind::Ret(_) => self.build_expr_stmt(block, expr),
        }
    }

    fn build_call(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
        lvalue: Lvalue<'tcx>,
        f: &tir::Expr<'tcx>,
        args: &[tir::Expr<'tcx>],
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        let f = set!(block = self.as_operand(block, f));
        let args = args.iter().map(|arg| set!(block = self.as_operand(block, arg))).collect_vec();
        let target = self.cfg.append_basic_block();
        self.cfg.terminate(
            info,
            block,
            TerminatorKind::Call { f, args, lvalue, target, unwind: None },
        );
        target.unit()
    }

    fn build_match(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
        lvalue: Lvalue<'tcx>,
        scrut: &tir::Expr<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) -> BlockAnd<()> {
        let info = self.span_info(expr.span);
        let scrut_rvalue = set!(block = self.as_rvalue(block, scrut));
        // TODO
        let (arm_blocks, default) = self.build_arms(lvalue, &scrut_rvalue, arms);

        // terminate all the switch blocks to branch back together again
        let end = self.cfg.append_basic_block();
        arm_blocks
            .iter()
            .for_each(|&(_, b)| self.cfg.terminate(info, b, TerminatorKind::Branch(end)));
        default.map(|id| self.cfg.terminate(info, id, TerminatorKind::Branch(end)));

        // if there is no default block, just create an unreachable one
        let default = default.unwrap_or_else(|| self.mk_unreachable(info));

        self.cfg.terminate(
            info,
            block,
            TerminatorKind::Switch { discr: scrut_rvalue, arms: arm_blocks, default },
        );
        end.unit()
    }

    fn mk_unreachable(&mut self, info: SpanInfo) -> BlockId {
        let block = self.cfg.append_basic_block();
        self.cfg.terminate(info, block, TerminatorKind::Unreachable);
        block
    }

    /// returns the switch arms and maybe the default block
    fn build_arms(
        &mut self,
        dest: Lvalue<'tcx>,
        scrut_rvalue: &Rvalue<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) -> (Vec<(Rvalue<'tcx>, BlockId)>, Option<BlockId>) {
        let mut switch_arms = Vec::with_capacity(arms.len());
        for arm in arms {
            let mut block = self.cfg.append_basic_block();
            set!(block = self.write_expr(block, dest, arm.body));
            // the first irrefutable pattern will be assigned the default block of the switch
            let rvalue = set!(block = self.build_arm_pat(block, arm.pat, scrut_rvalue));
            if !arm.pat.is_refutable() {
                return (switch_arms, Some(block));
            }
            switch_arms.push((rvalue, block));
        }
        (switch_arms, None)
    }

    fn build_arm_pat(
        &mut self,
        block: BlockId,
        pat: &tir::Pattern<'tcx>,
        cmp_rval: &Rvalue<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        match pat.kind {
            tir::PatternKind::Wildcard => block.and(cmp_rval.clone()),
            tir::PatternKind::Binding(ident, _) => {
                self.alloc_local(pat);
                block.and(cmp_rval.clone())
            }
            tir::PatternKind::Field(_) => todo!(),
            tir::PatternKind::Lit(c) => self.as_rvalue(block, c),
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

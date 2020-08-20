use crate::ast;
use crate::mir::build::*;
use crate::mir::*;
use crate::span::Span;
use crate::ty::Ty;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub(super) fn build_binary_op(
        &mut self,
        mut block: BlockId,
        span: Span,
        op: ast::BinOp,
        ty: Ty<'tcx>,
        lhs: Operand<'tcx>,
        rhs: Operand<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        block.and(Rvalue::Bin(op, lhs, rhs))
    }
}

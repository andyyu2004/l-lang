use lc_index::Idx;

use super::*;

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
    /// expr is handled in this function if there is a
    /// `Rvalue` variant corresponding directly to that expression
    pub fn as_rvalue(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        let info = self.span_info(expr.span);
        match expr.kind {
            tir::ExprKind::Adt { adt, variant_idx, substs, ref fields } => {
                // the fields passed to `Rvalue::Adt` must be in order of
                // `FieldIdx` specified in `adt.variants`
                // however, we evaluate the fields in the order specified by the user in `fields`
                let index_field_map: FxHashMap<FieldIdx, Operand<'tcx>> = fields
                    .iter()
                    .map(|f| (f.index, set!(block = self.as_operand(block, &f.expr))))
                    .collect();

                // creates the correctly ordered struct fields
                let fields = (0..adt.variants[variant_idx].fields.len())
                    .into_iter()
                    .map(FieldIdx::new)
                    .map(|idx| index_field_map[&idx])
                    .collect_vec();

                block.and(Rvalue::Adt { adt, variant_idx, substs, fields })
            }
            tir::ExprKind::Bin(op, ref l, ref r) => {
                let lhs = set!(block = self.as_operand(block, l));
                let rhs = set!(block = self.as_operand(block, r));
                self.build_binary_op(block, expr.span, expr.ty, op, lhs, rhs)
            }
            tir::ExprKind::Closure { ref upvars, ref body } =>
                self.build_closure(block, expr, upvars, body),
            tir::ExprKind::Unary(op, ref expr) => {
                let operand = set!(block = self.as_operand(block, &expr));
                block.and(Rvalue::Unary(op.into(), operand))
            }
            // assign is a bit out of place here,
            // as there is no direct rvalue variant for it
            // but it feels better than the other options so..
            tir::ExprKind::Assign(ref l, ref r) => {
                let rhs = set!(block = self.as_rvalue(block, r));
                let lhs = set!(block = self.as_lvalue(block, l));
                self.push_assignment(info, block, lhs, rhs.clone());
                block.and(rhs)
            }
            tir::ExprKind::Box(ref inner) => {
                let operand = set!(block = self.as_operand(block, inner));
                block.and(Rvalue::Box(operand))
            }
            tir::ExprKind::Ref(ref expr) => {
                let lvalue = set!(block = self.as_lvalue(block, &expr));
                block.and(Rvalue::Ref(lvalue))
            }
            tir::ExprKind::Block(..)
            | tir::ExprKind::ItemRef(..)
            | tir::ExprKind::Loop(..)
            | tir::ExprKind::Call(..)
            | tir::ExprKind::Match(..)
            | tir::ExprKind::Field(..)
            | tir::ExprKind::Tuple(..)
            | tir::ExprKind::Deref(_)
            | tir::ExprKind::Const(..)
            | tir::ExprKind::Ret(..)
            | tir::ExprKind::VarRef(..)
            | tir::ExprKind::Break
            | tir::ExprKind::Continue => {
                let operand = set!(block = self.as_operand(block, expr));
                block.and(Rvalue::Operand(operand))
            }
        }
    }

    pub(crate) fn build_binary_op(
        &mut self,
        block: BlockId,
        _span: Span,
        _ty: Ty<'tcx>,
        op: lc_ast::BinOp,
        lhs: Operand<'tcx>,
        rhs: Operand<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        // TODO some checks
        assert_eq!(self.operand_ty(lhs), self.operand_ty(rhs));
        block.and(Rvalue::Bin(op, lhs, rhs))
    }
}

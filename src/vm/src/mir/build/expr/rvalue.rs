use crate::ast;
use crate::mir::build::*;
use crate::mir::*;
use crate::set;
use crate::span::Span;
use crate::ty::Ty;
use itertools::Itertools;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    /// expr is handled in this function if there is a `Rvalue` variant corresponding to that expression
    pub fn as_rvalue(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        let info = self.span_info(expr.span);
        match expr.kind {
            tir::ExprKind::Adt { adt, variant_idx, substs, fields } => {
                // the fields passed to `Rvalue::Adt` must be in order of `FieldIdx` specified in `adt.variants`
                // however, we evaluate the fields in the order specified by the user in `fields`
                let index_field_map: FxHashMap<FieldIdx, Operand<'tcx>> = fields
                    .iter()
                    .map(|f| (f.index, set!(block = self.as_operand(block, f.expr))))
                    .collect();

                // creates the correctly ordered struct fields
                let fields = (0..adt.variants[variant_idx].fields.len())
                    .into_iter()
                    .map(FieldIdx::new)
                    .map(|idx| index_field_map[&idx])
                    .collect_vec();

                block.and(Rvalue::Adt { adt, variant_idx, substs, fields })
            }
            tir::ExprKind::Bin(op, l, r) => {
                let lhs = set!(block = self.as_operand(block, l));
                let rhs = set!(block = self.as_operand(block, r));
                self.build_binary_op(block, expr.span, expr.ty, op, lhs, rhs)
            }
            tir::ExprKind::Assign(l, r) => {
                let lhs = set!(block = self.as_lvalue(block, l));
                let rhs = set!(block = self.as_rvalue(block, r));
                self.push_assignment(info, block, lhs, rhs.clone());
                block.and(rhs)
            }
            tir::ExprKind::Tuple(xs) => block.and(Rvalue::Tuple(
                xs.iter().map(|x| set!(block = self.as_operand(block, x))).collect(),
            )),
            tir::ExprKind::Unary(_, _) => todo!(),
            tir::ExprKind::Block(_) => todo!(),
            tir::ExprKind::ItemRef(_) => todo!(),
            tir::ExprKind::Lambda(_) => todo!(),
            tir::ExprKind::Call(_, _) => todo!(),
            tir::ExprKind::Match(_, _) => todo!(),
            // forward the implementation to `as_operand` if the expr is in some sense "atomic"
            tir::ExprKind::Field(..)
            | tir::ExprKind::Const(..)
            | tir::ExprKind::VarRef(..)
            | tir::ExprKind::Ret(..) => {
                let operand = set!(block = self.as_operand(block, expr));
                block.and(Rvalue::Use(operand))
            }
        }
    }

    pub(super) fn build_binary_op(
        &mut self,
        block: BlockId,
        span: Span,
        ty: Ty<'tcx>,
        op: ast::BinOp,
        lhs: Operand<'tcx>,
        rhs: Operand<'tcx>,
    ) -> BlockAnd<Rvalue<'tcx>> {
        // TODO some checks
        block.and(Rvalue::Bin(op, lhs, rhs))
    }
}

use crate::ast;
use crate::mir::build::*;
use crate::mir::*;
use crate::set;
use crate::span::Span;
use crate::ty::Ty;

/// helper struct for building projections of an lvalue
pub struct LvalueBuilder<'tcx> {
    id: VarId,
    projs: Vec<Projection<'tcx>>,
}

impl<'tcx> LvalueBuilder<'tcx> {
    pub fn lvalue(self, tcx: TyCtx<'tcx>) -> Lvalue<'tcx> {
        Lvalue { id: self.id, projs: tcx.intern_lvalue_projections(&self.projs) }
    }
}
impl<'tcx> From<VarId> for LvalueBuilder<'tcx> {
    fn from(id: VarId) -> Self {
        Self { id, projs: vec![] }
    }
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn as_lvalue(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<Lvalue<'tcx>> {
        let builder = set!(block = self.as_lvalue_builder(block, expr));
        block.and(builder.lvalue(self.tcx))
    }

    pub fn as_lvalue_builder(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<LvalueBuilder<'tcx>> {
        match expr.kind {
            tir::ExprKind::VarRef(id) => block.and(LvalueBuilder::from(self.var_ir_map[&id])),
            tir::ExprKind::Const(_)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Unary(..)
            | tir::ExprKind::Block(..)
            | tir::ExprKind::ItemRef(..)
            | tir::ExprKind::Tuple(..)
            | tir::ExprKind::Lambda(..)
            | tir::ExprKind::Call(..)
            | tir::ExprKind::Match(..)
            | tir::ExprKind::Assign(..)
            | tir::ExprKind::Ret(..) => {
                // if the expr is not an lvalue, create a temporary
                let var = set!(block = self.as_tmp(block, expr));
                block.and(var.into())
            }
        }
    }
}

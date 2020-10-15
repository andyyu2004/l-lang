use super::*;
use lcore::ty::{Ty, TyCtx};

/// helper struct for building projections of an lvalue
pub struct LvalueBuilder<'tcx> {
    id: VarId,
    projs: Vec<Projection<'tcx>>,
}

impl<'tcx> LvalueBuilder<'tcx> {
    pub fn lvalue(self, tcx: TyCtx<'tcx>) -> Lvalue<'tcx> {
        Lvalue { id: self.id, projs: tcx.intern_lvalue_projections(&self.projs) }
    }

    pub fn project(mut self, proj: Projection<'tcx>) -> Self {
        self.projs.push(proj);
        self
    }

    pub fn project_cast(self, ty: Ty<'tcx>) -> Self {
        self.project(Projection::PointerCast(ty))
    }

    pub fn project_deref(self) -> Self {
        self.project(Projection::Deref)
    }

    pub fn project_field(self, field: FieldIdx, ty: Ty<'tcx>) -> Self {
        self.project(Projection::Field(field, ty))
    }
}

impl<'tcx> From<Lvalue<'tcx>> for LvalueBuilder<'tcx> {
    fn from(lvalue: Lvalue<'tcx>) -> Self {
        Self { id: lvalue.id, projs: lvalue.projs.to_vec() }
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

    pub fn var_id_as_lvalue_builder(&mut self, id: ir::Id) -> LvalueBuilder<'tcx> {
        if let Some(&var_id) = self.var_ir_map.get(&id) {
            LvalueBuilder::from(var_id)
        } else {
            panic!("no var found with id `{}`", id)
        }
    }

    pub fn as_lvalue_builder(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<LvalueBuilder<'tcx>> {
        match expr.kind {
            tir::ExprKind::VarRef(id) => block.and(self.var_id_as_lvalue_builder(id)),
            tir::ExprKind::Field(ref base, idx) => {
                let builder = set!(block = self.as_lvalue_builder(block, base));
                block.and(builder.project_field(idx, expr.ty))
            }
            tir::ExprKind::Deref(ref expr) => {
                let builder = set!(block = self.as_lvalue_builder(block, expr));
                block.and(builder.project_deref())
            }
            tir::ExprKind::Const(_)
            | tir::ExprKind::Bin(..)
            | tir::ExprKind::Ref(..)
            | tir::ExprKind::Box(..)
            | tir::ExprKind::Adt { .. }
            | tir::ExprKind::Unary(..)
            | tir::ExprKind::Block(..)
            | tir::ExprKind::ItemRef(..)
            | tir::ExprKind::Tuple(..)
            | tir::ExprKind::Closure { .. }
            | tir::ExprKind::Call(..)
            | tir::ExprKind::Match(..)
            | tir::ExprKind::Assign(..)
            | tir::ExprKind::Ret(..) => {
                // if the expr is not an lvalue, create a temporary and return that as an lvalue
                let var = set!(block = self.as_tmp(block, expr));
                block.and(var.into())
            }
        }
    }
}

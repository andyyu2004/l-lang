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

    pub fn project(mut self, proj: Projection<'tcx>) -> Self {
        self.projs.push(proj);
        self
    }

    pub fn project_deref(self) -> Self {
        self.project(Projection::Deref)
    }

    pub fn project_field(self, field: FieldIdx, ty: Ty<'tcx>) -> Self {
        self.project(Projection::Field(field, ty))
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

    pub fn var_id_as_lvalue(&mut self, id: ir::Id) -> Lvalue<'tcx> {
        self.var_id_as_lvalue_builder(id).lvalue(self.tcx)
    }

    pub fn var_id_as_lvalue_builder(&mut self, id: ir::Id) -> LvalueBuilder<'tcx> {
        let var_id = self
            .var_ir_map
            .get(&id)
            .copied()
            .unwrap_or_else(|| panic!("no variable with id `{}`", id));
        let var = self.vars[var_id];
        match var.kind {
            // there needs to be an implicit dereference on upvars
            // as every upvar is actually a mutable reference to the captured variable
            VarKind::Upvar => LvalueBuilder::from(var_id).project_deref(),
            _ => LvalueBuilder::from(var_id),
        }
    }

    pub fn as_lvalue_builder(
        &mut self,
        mut block: BlockId,
        expr: &tir::Expr<'tcx>,
    ) -> BlockAnd<LvalueBuilder<'tcx>> {
        match expr.kind {
            tir::ExprKind::VarRef(id) => block.and(self.var_id_as_lvalue_builder(id)),
            tir::ExprKind::Field(base, idx) => {
                let builder = set!(block = self.as_lvalue_builder(block, base));
                block.and(builder.project_field(idx, expr.ty))
            }
            tir::ExprKind::Deref(expr) => {
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

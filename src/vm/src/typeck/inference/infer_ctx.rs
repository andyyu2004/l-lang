use super::*;
use crate::ir::DefId;
use crate::typeck::{TyCtx, TypeckTables};
use crate::{ast, ir, tir, ty::Ty};
use std::cell::RefCell;

crate struct InferCtxBuilder<'tcx> {
    /// `DefId` of the item being typechecked
    def_id: DefId,
    tcx: TyCtx<'tcx>,
    tables: RefCell<TypeckTables<'tcx>>,
}

impl<'tcx> InferCtxBuilder<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, def_id: DefId) -> Self {
        Self { tcx, def_id, tables: RefCell::new(TypeckTables::new(def_id)) }
    }

    pub fn enter<R>(&mut self, f: impl for<'a> FnOnce(InferCtx<'a, 'tcx>) -> R) -> R {
        f(InferCtx::new(self.tcx, &self.tables))
    }
}

#[derive(Default)]
crate struct InferCtxInner<'tcx> {
    type_variable_storage: TypeVariableStorage<'tcx>,
    undo_log: InferCtxUndoLogs<'tcx>,
}

impl<'tcx> InferCtxInner<'tcx> {
    pub fn type_variables(&mut self) -> TypeVariableTable<'_, 'tcx> {
        self.type_variable_storage.with_log(&mut self.undo_log)
    }
}

crate struct InferCtx<'a, 'tcx> {
    pub tcx: TyCtx<'tcx>,
    pub inner: RefCell<InferCtxInner<'tcx>>,
    tables: &'a RefCell<TypeckTables<'tcx>>,
}

impl<'a, 'tcx> InferCtx<'a, 'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, tables: &'a RefCell<TypeckTables<'tcx>>) -> Self {
        Self { tcx, tables, inner: Default::default() }
    }

    pub fn check_fn(
        &'a self,
        item: &ir::Item,
        sig: &ir::FnSig,
        generics: &ir::Generics,
        body: &ir::Body,
    ) -> FnCtx<'a, 'tcx> {
        let fcx = FnCtx::new(&self);
        let (_, ret_ty) = self.tcx.item_ty(item.id.def_id).expect_fn();
        let body_ty = fcx.check_expr(body.expr);
        fcx.expect_eq(item.span, ret_ty, body_ty);
        fcx
    }

    pub fn node_ty(&self, id: ir::Id) -> Ty<'tcx> {
        *self.tables.borrow().node_types().get(id).expect("no entry for id in typecktables")
    }

    pub fn write_ty(&self, id: ir::Id, ty: Ty<'tcx>) {
        self.tables.borrow_mut().node_types_mut().insert(id, ty);
    }
}

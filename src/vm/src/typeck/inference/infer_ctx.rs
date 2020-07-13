use super::*;
use crate::ir::DefId;
use crate::ty::{InferTy, SubstRef, Ty, TyConv, TyKind};
use crate::typeck::{TyCtx, TypeckTables};
use crate::{ast, error::TypeResult, ir, tir};
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

    /// creates the substitutions for the inference variables
    pub fn inference_substs(&self) -> TypeResult<'tcx, SubstRef<'tcx>> {
        let vec = self.inner.borrow_mut().type_variables().gen_substs()?;
        let substs = self.tcx.intern_substs(&vec);
        Ok(substs)
    }

    /// top level entry point for typechecking a function item
    pub fn check_fn(
        &'a self,
        item: &ir::Item,
        sig: &ir::FnSig,
        generics: &ir::Generics,
        body: &ir::Body,
    ) -> FnCtx<'a, 'tcx> {
        let mut fcx = FnCtx::new(&self);
        let (_, ret_ty) = self.tcx.item_ty(item.id.def_id).expect_fn();
        let body_ty = fcx.check_expr(body.expr);
        info!("body type: {}; ret_ty: {}", body_ty, ret_ty);
        fcx.expect_eq(item.span, ret_ty, body_ty);
        fcx
    }

    /// create new type inference variable
    pub fn new_infer_var(&self) -> Ty<'tcx> {
        let vid = self.inner.borrow_mut().type_variables().new_ty_var();
        self.tcx.mk_ty(TyKind::Infer(InferTy::TyVar(vid)))
    }

    pub fn node_ty(&self, id: ir::Id) -> Ty<'tcx> {
        self.tables.borrow().node_type(id)
    }

    pub fn lower_ty(&self, ir_ty: &ir::Ty) -> Ty<'tcx> {
        TyConv::ir_ty_to_ty(self, ir_ty)
    }

    /// records the type for the given id in the tables
    /// returns the same type purely for convenience
    pub fn write_ty(&self, id: ir::Id, ty: Ty<'tcx>) -> Ty<'tcx> {
        self.tables.borrow_mut().node_types_mut().insert(id, ty);
        ty
    }
}

impl<'a, 'tcx> TyConv<'tcx> for InferCtx<'a, 'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        self.tcx
    }
}

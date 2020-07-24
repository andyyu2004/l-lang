use super::*;
use crate::error::{DiagnosticBuilder, TypeError, TypeResult};
use crate::ir::DefId;
use crate::span::Span;
use crate::ty::{InferTy, InferenceVarSubstFolder, SubstRef, Ty, TyConv, TyKind, TypeFoldable};
use crate::typeck::{TyCtx, TypeckTables};
use crate::{ast, ir, tir};
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

    pub fn unify(&self, span: Span, expected: Ty<'tcx>, actual: Ty<'tcx>) {
        self.unify_diag(span, expected, actual).unwrap_or_else(|err| {
            err.emit();
            self.tcx.mk_ty_err()
        });
    }

    fn unify_diag(
        &self,
        span: Span,
        expected: Ty<'tcx>,
        actual: Ty<'tcx>,
    ) -> Result<Ty<'tcx>, DiagnosticBuilder> {
        match self.at(span).equate(expected, actual) {
            Ok(ty) => Ok(ty),
            Err(err) => Err(self.report_type_error(span, err)),
        }
    }

    fn report_type_error(&self, span: Span, err: TypeError<'tcx>) -> DiagnosticBuilder {
        DiagnosticBuilder::from_err(span, err)
    }

    /// creates the substitutions for the inference variables
    pub fn inference_substs(&self) -> TypeResult<'tcx, SubstRef<'tcx>> {
        let vec = self.inner.borrow_mut().type_variables().gen_substs()?;
        let mut substs = self.tcx.intern_substs(&vec);
        // repeatedly substitutes its inference variables value
        // until it contains no inference variables or failure
        // I think this process won't/can't be cyclic?
        let mut folder = InferenceVarSubstFolder::new(self.tcx, substs);
        loop {
            let new_substs = substs.fold_with(&mut folder);
            if substs == new_substs {
                break;
            }
            substs = new_substs;
        }
        debug_assert!(substs.iter().all(|ty| !ty.has_infer_vars()));
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
        let (param_tys, ret_ty) = self.tcx.item_ty(item.id.def_id).expect_fn();
        let body_ty = fcx.check_body(param_tys, body);
        info!("body type: {}; ret_ty: {}", body_ty, ret_ty);
        fcx.unify(item.span, ret_ty, body_ty);
        fcx
    }

    /// create new type inference variable
    pub fn new_infer_var(&self) -> Ty<'tcx> {
        let vid = self.inner.borrow_mut().type_variables().new_ty_var();
        self.tcx.mk_ty(TyKind::Infer(InferTy::TyVar(vid)))
    }

    pub fn node_ty(&self, id: ir::Id) -> Ty<'tcx> {
        info!("fcx query node type for {:?}", id);
        self.tables.borrow().node_type(id)
    }

    /// records the type for the given id in the tables
    /// returns the same type purely for convenience
    pub fn write_ty(&self, id: ir::Id, ty: Ty<'tcx>) -> Ty<'tcx> {
        info!("fcx write ty {:?} : {}", id, ty);
        self.tables.borrow_mut().node_types_mut().insert(id, ty);
        ty
    }
}

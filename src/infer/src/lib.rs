#![feature(crate_visibility_modifier)]

#[macro_use]
extern crate log;

mod at;
mod equate;
mod instantiate;
mod type_variables;

use at::At;
use equate::Equate;
use error::{DiagnosticBuilder, MultiSpan};
use index::Idx;
use instantiate::InstantiationFolder;
use ir::{DefId, FieldIdx};
use lcore::ty::*;
use span::Span;
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::ops::Deref;
use type_variables::*;

pub struct InferCtxBuilder<'tcx> {
    tcx: TyCtx<'tcx>,
    tables: RefCell<TypeckTables<'tcx>>,
}

impl<'tcx> InferCtxBuilder<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, def_id: DefId) -> Self {
        Self { tcx, tables: RefCell::new(TypeckTables::new(def_id)) }
    }

    pub fn enter<R>(&mut self, f: impl for<'a> FnOnce(InferCtx<'a, 'tcx>) -> R) -> R {
        f(InferCtx::new(self.tcx, &self.tables))
    }
}

#[derive(Default)]
pub struct InferCtxInner<'tcx> {
    type_variable_storage: TypeVariableStorage<'tcx>,
    undo_log: InferCtxUndoLogs<'tcx>,
}

impl<'tcx> InferCtxInner<'tcx> {
    pub fn type_variables(&mut self) -> TypeVariableTable<'_, 'tcx> {
        self.type_variable_storage.with_log(&mut self.undo_log)
    }
}

pub struct InferCtx<'a, 'tcx> {
    pub tcx: TyCtx<'tcx>,
    pub inner: RefCell<InferCtxInner<'tcx>>,
    pub tables: &'a RefCell<TypeckTables<'tcx>>,
    has_error: Cell<bool>,
}

impl<'tcx> Deref for InferCtx<'_, 'tcx> {
    type Target = TyCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.tcx
    }
}

impl<'a, 'tcx> InferCtx<'a, 'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, tables: &'a RefCell<TypeckTables<'tcx>>) -> Self {
        Self { tcx, tables, has_error: Cell::new(false), inner: Default::default() }
    }

    /// equates two types in the unification table
    /// emits a type error if the types are not unifiable
    pub fn equate(&self, span: Span, expected: Ty<'tcx>, ty: Ty<'tcx>) {
        if expected.contains_err() || ty.contains_err() {
            return;
        }
        if let Err(err) = self.at(span).equate(expected, ty) {
            self.emit_ty_err(span, err);
        }
    }

    /// attempts to get the dereferenced type of `ty`
    pub fn deref_ty(&self, span: Span, ty: Ty<'tcx>) -> Ty<'tcx> {
        let ty = self.partially_resolve_ty(span, ty);
        match ty.kind {
            TyKind::Box(_, ty) | TyKind::Ptr(ty) => ty,
            _ => self.emit_ty_err(span, TypeError::InvalidDereference(ty)),
        }
    }

    pub fn emit_ty_err(&self, span: impl Into<MultiSpan>, err: impl Error) -> Ty<'tcx> {
        let diag = self.tcx.sess.build_error(span, err);
        self.emit_err(diag)
    }

    pub fn emit_err(&self, err: DiagnosticBuilder) -> Ty<'tcx> {
        err.emit();
        self.set_ty_err()
    }

    /// creates a type error and sets the error flag
    pub fn set_ty_err(&self) -> Ty<'tcx> {
        self.has_error.set(true);
        self.tcx.mk_ty_err()
    }

    /// creates the conrete substitutions for all inference variables
    pub fn inference_substs(&self) -> SubstsRef<'tcx> {
        // let vec: Vec<_> = self.inner.borrow_mut().type_variables();
        let mut inner = self.inner.borrow_mut();
        let mut type_variables = inner.type_variables();
        // generates an indexed substitution based on the contents of the UnificationTable
        let mut substs = self.tcx.mk_substs((0..type_variables.storage.tyvid_count).map(|index| {
            let vid = TyVid { index };
            let val = type_variables.probe(vid);
            match val {
                TyVarValue::Known(ty) => type_variables.instantiate_if_known(ty),
                TyVarValue::Unknown => {
                    let span = type_variables.storage.tyvar_data[&vid].span;
                    self.emit_ty_err(span, TypeError::InferenceFailure)
                }
            }
        }));

        // repeatedly substitute its inference variables for its value
        // until it contains no inference variables or failure
        // I think this will always terminate?
        let mut folder = InferVarSubstsFolder::new(self.tcx, substs);
        loop {
            let new_substs = substs.fold_with(&mut folder);
            if substs == new_substs {
                break;
            }
            substs = new_substs;
        }
        assert!(substs.iter().all(|ty| !ty.has_infer_vars()));
        substs
    }

    /// if `ty` is an inference variable, attempts to resolve it at least one level deep
    pub fn partially_resolve_ty(&self, span: Span, ty: Ty<'tcx>) -> Ty<'tcx> {
        match ty.kind {
            TyKind::Infer(infer) =>
                self.partially_resolve_ty(span, self.resolve_infer_var(span, infer)),
            _ => ty,
        }
    }

    /// returns the concrete type for a type variable and reports an error if it is unknown
    pub fn resolve_infer_var(&self, span: Span, infer: InferTy) -> Ty<'tcx> {
        let mut inner = self.inner.borrow_mut();
        let mut tyvars = inner.type_variables();
        let value = match infer {
            TyVar(tyvid) => tyvars.probe(tyvid),
        };
        match value {
            TyVarValue::Known(ty) => ty,
            TyVarValue::Unknown => self.emit_ty_err(span, TypeError::InferenceFailure),
        }
    }

    pub fn instantiate(&self, span: Span, ty: Ty<'tcx>) -> Ty<'tcx> {
        match &ty.kind {
            TyKind::Scheme(forall, ty) =>
                ty.fold_with(&mut InstantiationFolder::new(self, span, forall)),
            _ => ty,
        }
    }

    /// create new type inference variable
    pub fn new_infer_var(&self, span: Span) -> Ty<'tcx> {
        let vid = self.inner.borrow_mut().type_variables().new_ty_var(span);
        self.tcx.mk_ty(TyKind::Infer(InferTy::TyVar(vid)))
    }

    pub fn node_ty(&self, id: ir::Id) -> Ty<'tcx> {
        debug!("fcx query node type for {:?}", id);
        self.tables.borrow().node_type(id)
    }

    /// records the type for the given id in the tables
    /// returns the same type purely for convenience
    pub fn record_ty(&self, id: ir::Id, ty: Ty<'tcx>) -> Ty<'tcx> {
        debug!("fcx write ty {:?} : {}", id, ty);
        self.tables.borrow_mut().node_types_mut().insert(id, ty);
        ty
    }

    pub fn record_adjustments(&self, id: ir::Id, adjustments: Vec<Adjustment<'tcx>>) {
        self.tables.borrow_mut().adjustments_mut().insert(id, adjustments);
    }

    pub fn record_field_index(&self, id: ir::Id, idx: usize) {
        debug!("fcx write field_index {:?} : {}", id, idx);
        self.tables.borrow_mut().field_indices_mut().insert(id, FieldIdx::new(idx));
    }
}

pub trait TyCtxtInferExt<'tcx> {
    fn infer_ctx(self, def_id: DefId) -> InferCtxBuilder<'tcx>;
}

impl<'tcx> TyCtxtInferExt<'tcx> for TyCtx<'tcx> {
    fn infer_ctx(self, def_id: DefId) -> InferCtxBuilder<'tcx> {
        InferCtxBuilder::new(self, def_id)
    }
}

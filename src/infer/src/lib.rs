#![feature(crate_visibility_modifier)]

#[macro_use]
extern crate log;

mod at;
mod equate;
mod snapshot;
mod type_variables;
mod undo;

use at::At;
use equate::Equate;
use error::{DiagnosticBuilder, MultiSpan};
use index::Idx;
use ir::{DefId, FieldIdx, Res};
use lcore::ty::*;
use snapshot::*;
use span::Span;
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::ops::Deref;
use type_variables::*;
use undo::*;

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
    undo_logs: InferCtxUndoLogs<'tcx>,
}

impl<'tcx> InferCtxInner<'tcx> {
    pub fn type_variables(&mut self) -> TypeVariableTable<'_, 'tcx> {
        self.type_variable_storage.with_log(&mut self.undo_logs)
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

    /// execute `f` then undo any variables it creates
    /// used to `probe` and try things that we don't want to persist
    pub fn probe<R>(&self, f: impl FnOnce(&InferCtxSnapshot<'a, 'tcx>) -> R) -> R {
        let snapshot = self.start_snapshot();
        let r = f(&snapshot);
        self.rollback_to(snapshot);
        r
    }

    /// equates two types in the unification table
    /// emits a type error if the types are not unifiable
    pub fn unify(&self, span: Span, expected: Ty<'tcx>, ty: Ty<'tcx>) {
        if expected.contains_err() || ty.contains_err() {
            return;
        }

        if let TypeResult::Err(err) = self.at(span).equate(expected, ty) {
            self.emit_ty_err(span, err);
        }
    }

    /// attempts to get the dereferenced type of `ty`
    pub fn deref_ty(&self, span: Span, ty: Ty<'tcx>) -> Ty<'tcx> {
        let ty = self.partially_resolve_ty(span, ty);
        match ty.kind {
            TyKind::Box(ty) | TyKind::Ptr(ty) => ty,
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
        let mut substs =
            self.tcx.mk_substs((0..type_variables.storage.tyvar_data.len()).map(|index| {
                let vid = TyVid { index: index as u32 };
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

    /// instantiates the item with def_id, and records the substitutions
    pub fn instantiate(&self, xpat: &dyn ir::ExprOrPat<'tcx>, def_id: DefId) -> Ty<'tcx> {
        let ty = self.type_of(def_id);
        match ty.kind {
            TyKind::Adt(..) | TyKind::FnPtr(..) => {}
            _ => unreachable!("not instantiable"),
        };
        let generics = self.generics_of(def_id);
        let substs =
            self.mk_substs(generics.params.iter().map(|_| self.new_infer_var(xpat.span())));
        self.record_substs(xpat.id(), substs);
        ty.subst(self.tcx, substs)
    }

    /// create a fresh type inference variable
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
        debug!("record ty {:?} : {}", id, ty);
        // sometimes we deliberately overwrite the type for a given id
        // e.g. in `check_body` so we don't perform the assertion
        self.tables.borrow_mut().node_types_mut().insert(id, ty);
        ty
    }

    pub fn record_substs(&self, id: ir::Id, substs: SubstsRef<'tcx>) {
        debug!("record substs {:?} : {}", id, substs);
        assert!(self.tables.borrow_mut().node_substs_mut().insert(id, substs).is_none());
    }

    pub fn record_type_relative_res(&self, id: ir::Id, res: Res) {
        debug!("record type relative res {:?} : {}", id, res);
        assert!(self.tables.borrow_mut().type_relative_resolutions_mut().insert(id, res).is_none());
    }

    pub fn record_adjustments(&self, id: ir::Id, adjustments: Vec<Adjustment<'tcx>>) {
        debug!("record adjustments {:?} : {:?}", id, adjustments);
        assert!(self.tables.borrow_mut().adjustments_mut().insert(id, adjustments).is_none());
    }

    pub fn record_field_index(&self, id: ir::Id, idx: usize) {
        debug!("record field_index {:?} : {}", id, idx);
        assert!(
            self.tables.borrow_mut().field_indices_mut().insert(id, FieldIdx::new(idx)).is_none()
        );
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

impl<'a, 'tcx> InferCtx<'a, 'tcx> {
    pub fn fresh_substs_for_item(&self, def_id: DefId) -> SubstsRef<'tcx> {
        let generics = self.generics_of(def_id);
        let span = self.defs().span(def_id);
        let params = generics.params.iter().map(|_| self.new_infer_var(span));
        self.mk_substs(params)
    }
}

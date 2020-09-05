use super::*;
use crate::error::{DiagnosticBuilder, TypeError, TypeResult};
use crate::ir::DefId;
use crate::span::Span;
use crate::ty::*;
use crate::typeck::{TyCtx, TypeckOutputs};
use crate::{ast, ir, tir};
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::ops::Deref;

pub struct InferCtxBuilder<'tcx> {
    /// `DefId` of the item being typechecked
    def_id: DefId,
    tcx: TyCtx<'tcx>,
    tables: RefCell<TypeckOutputs<'tcx>>,
}

impl<'tcx> InferCtxBuilder<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, def_id: DefId) -> Self {
        Self { tcx, def_id, tables: RefCell::new(TypeckOutputs::new(def_id)) }
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
    tables: &'a RefCell<TypeckOutputs<'tcx>>,
    has_error: Cell<bool>,
}

impl<'tcx> Deref for InferCtx<'_, 'tcx> {
    type Target = TyCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.tcx
    }
}

impl<'a, 'tcx> InferCtx<'a, 'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, tables: &'a RefCell<TypeckOutputs<'tcx>>) -> Self {
        Self { tcx, tables, has_error: Cell::new(false), inner: Default::default() }
    }

    pub fn unify(&self, span: Span, expected: Ty<'tcx>, actual: Ty<'tcx>) {
        if expected.contains_err() || actual.contains_err() {
            return;
        }
        if let Err(err) = self.at(span).equate(expected, actual) {
            self.emit_ty_err(span, err);
        }
    }

    pub fn emit_ty_err(&self, span: Span, err: impl Error) -> Ty<'tcx> {
        let diag = self.tcx.session.build_error(span, err);
        self.emit_err(diag)
    }

    pub fn emit_err(&self, err: DiagnosticBuilder) -> Ty<'tcx> {
        err.emit();
        self.report_ty_err()
    }

    pub fn report_ty_err(&self) -> Ty<'tcx> {
        self.has_error.set(true);
        self.tcx.mk_ty_err()
    }

    /// creates the substitutions for the inference variables
    pub fn inference_substs(&self) -> SubstsRef<'tcx> {
        let vec: Vec<_> = self.inner.borrow_mut().type_variables().gen_substs(self);
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
        substs
    }

    pub fn instantiate(&self, span: Span, ty: Ty<'tcx>) -> Ty<'tcx> {
        match &ty.kind {
            TyKind::Scheme(forall, ty) => {
                let mut folder = InstantiationFolder::new(self, span, forall);
                ty.fold_with(&mut folder)
            }
            _ => ty,
        }
    }

    /// create new type inference variable
    pub fn new_infer_var(&self, span: Span) -> Ty<'tcx> {
        let vid = self.inner.borrow_mut().type_variables().new_ty_var(span);
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

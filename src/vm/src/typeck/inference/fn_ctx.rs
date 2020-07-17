use super::InferCtx;
use crate::error::TypeResult;
use crate::ir::{self, DefId};
use crate::span::Span;
use crate::tir;
use crate::ty::{SubstRef, Ty, TyConv, TyKind};
use crate::typeck::{TyCtx, TypeckTables};
use ir::FnSig;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, ops::Deref};

crate struct FnCtx<'a, 'tcx> {
    infcx: &'a InferCtx<'a, 'tcx>,
    locals: FxHashMap<ir::Id, Ty<'tcx>>,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn new(infcx: &'a InferCtx<'a, 'tcx>) -> Self {
        Self { infcx, locals: Default::default() }
    }
}

impl<'a, 'tcx> Deref for FnCtx<'a, 'tcx> {
    type Target = InferCtx<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.infcx
    }
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn unify(&self, span: Span, expected: Ty<'tcx>, actual: Ty<'tcx>) {
        // handle and report the error here
        if let Err(err) = self.at(span).equate(expected, actual) {
            println!("{}", err)
        }
    }

    pub fn lower_tys(&self, ir_tys: &[ir::Ty]) -> &'tcx [Ty<'tcx>] {
        self.tcx.mk_substs(ir_tys.iter().map(|ty| TyConv::ir_ty_to_ty(self, ty)))
    }

    pub fn lower_ty(&self, ir_ty: &ir::Ty) -> Ty<'tcx> {
        TyConv::ir_ty_to_ty(self, ir_ty)
    }

    pub fn def_local(&mut self, id: ir::Id, ty: Ty<'tcx>) -> Ty<'tcx> {
        info!("deflocal {:?} : {}", id, ty);
        self.locals.insert(id, ty);
        ty
    }

    pub fn local_ty(&self, id: ir::Id) -> Ty<'tcx> {
        info!("lookup ty for local {:?}", id);
        self.locals.get(&id).cloned().expect("no entry for local variable")
    }
}

impl<'a, 'tcx> TyConv<'tcx> for FnCtx<'a, 'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        self.tcx
    }

    fn infer_ty(&self, span: Span) -> Ty<'tcx> {
        self.infcx.new_infer_var()
    }
}

use super::{InferCtx, InferCtxBuilder};
use crate::error::{DiagnosticBuilder, TypeResult};
use crate::ir::{self, DefId};
use crate::span::Span;
use crate::tir;
use crate::ty::{SubstRef, Ty, TyConv, TyKind};
use crate::typeck::{TyCtx, TypeckTables};
use ir::FnSig;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, ops::Deref};

pub struct FnCtx<'a, 'tcx> {
    inherited: &'a Inherited<'a, 'tcx>,
    pub(super) expected_ret_ty: Ty<'tcx>,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn new(inherited: &'a Inherited<'a, 'tcx>) -> Self {
        Self { inherited, expected_ret_ty: inherited.new_infer_var() }
    }
}

impl<'a, 'tcx> Deref for FnCtx<'a, 'tcx> {
    type Target = Inherited<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.inherited
    }
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn lower_tys(&self, ir_tys: &[ir::Ty]) -> &'tcx [Ty<'tcx>] {
        self.tcx.mk_substs(ir_tys.iter().map(|ty| TyConv::ir_ty_to_ty(self.infcx, ty)))
    }

    pub fn lower_ty(&self, ir_ty: &ir::Ty) -> Ty<'tcx> {
        TyConv::ir_ty_to_ty(self.infcx, ir_ty)
    }
}

impl<'a, 'tcx> TyConv<'tcx> for InferCtx<'a, 'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        self.tcx
    }

    fn infer_ty(&self, span: Span) -> Ty<'tcx> {
        self.new_infer_var()
    }
}

impl<'a, 'tcx> Deref for Inherited<'a, 'tcx> {
    type Target = InferCtx<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.infcx
    }
}

/// stuff that is shared between functions
/// nested lambdas will have their own `FnCtx` but will share `Inherited` will outer lambdas as
/// well as the outermost fn item
pub struct Inherited<'a, 'tcx> {
    infcx: &'a InferCtx<'a, 'tcx>,
    locals: RefCell<FxHashMap<ir::Id, Ty<'tcx>>>,
}

pub struct InheritedBuilder<'tcx> {
    infcx: InferCtxBuilder<'tcx>,
}

impl<'tcx> InheritedBuilder<'tcx> {
    pub fn enter<R>(&mut self, f: impl for<'a> FnOnce(Inherited<'a, 'tcx>) -> R) -> R {
        self.infcx.enter(|infcx| f(Inherited::new(&infcx)))
    }
}

impl<'a, 'tcx> Inherited<'a, 'tcx> {
    pub fn new(infcx: &'a InferCtx<'a, 'tcx>) -> Self {
        Self { infcx, locals: Default::default() }
    }

    pub fn build(tcx: TyCtx<'tcx>, def_id: DefId) -> InheritedBuilder<'tcx> {
        InheritedBuilder { infcx: tcx.infer_ctx(def_id) }
    }

    /// top level entry point for typechecking a function item
    pub fn check_fn_item(
        &'a self,
        item: &ir::Item,
        sig: &ir::FnSig,
        generics: &ir::Generics,
        body: &ir::Body,
    ) -> FnCtx<'a, 'tcx> {
        let fn_ty = self.tcx.item_ty(item.id.def);
        // don't instantiate anything and typeck the body using the param tys
        // don't know if this is a good idea
        let (_forall, ty) = fn_ty.expect_scheme();
        debug_assert_eq!(ty, TyConv::fn_sig_to_ty(self.infcx, sig));
        let (param_tys, ret_ty) = ty.expect_fn();
        self.check_fn(sig, body).0
    }

    pub fn check_fn(&'a self, sig: &ir::FnSig, body: &ir::Body) -> (FnCtx<'a, 'tcx>, Ty<'tcx>) {
        let fn_ty = TyConv::fn_sig_to_ty(self.infcx, sig);
        let mut fcx = FnCtx::new(self);
        let body_ty = fcx.check_body(fn_ty, body);
        (fcx, fn_ty)
    }

    pub fn def_local(&self, id: ir::Id, ty: Ty<'tcx>) -> Ty<'tcx> {
        info!("deflocal {:?} : {}", id, ty);
        self.locals.borrow_mut().insert(id, ty);
        ty
    }

    pub fn local_ty(&self, id: ir::Id) -> Ty<'tcx> {
        info!("lookup ty for local {:?}", id);
        self.locals.borrow().get(&id).cloned().expect("no entry for local variable")
    }
}

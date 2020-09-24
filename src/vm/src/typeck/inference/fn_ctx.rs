use super::{InferCtx, InferCtxBuilder};
use crate::ast::Mutability;
use crate::error::{DiagnosticBuilder, TypeError, TypeResult};
use crate::ir::{self, DefId, FnSig};
use crate::lexer::symbol;
use crate::span::Span;
use crate::tir;
use crate::ty::{SubstsRef, Ty, TyConv, TyKind};
use crate::typeck::{TyCtx, TypeckOutputs};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::ops::Deref;

pub struct FnCtx<'a, 'tcx> {
    inherited: &'a InheritedCtx<'a, 'tcx>,
    pub(super) fn_ty: Ty<'tcx>,
    pub(super) param_tys: SubstsRef<'tcx>,
    pub(super) ret_ty: Ty<'tcx>,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn new(inherited: &'a InheritedCtx<'a, 'tcx>, fn_ty: Ty<'tcx>) -> Self {
        let (param_tys, ret_ty) = fn_ty.expect_fn();
        Self { inherited, fn_ty, param_tys, ret_ty }
    }
}

impl<'a, 'tcx> Deref for FnCtx<'a, 'tcx> {
    type Target = InheritedCtx<'a, 'tcx>;

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
        self.new_infer_var(span)
    }
}

impl<'a, 'tcx> Deref for InheritedCtx<'a, 'tcx> {
    type Target = InferCtx<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.infcx
    }
}

/// context that is shared between functions
/// nested lambdas will have their own `FnCtx` but will share `Inherited` will outer lambdas as
/// well as the outermost fn item
pub struct InheritedCtx<'a, 'tcx> {
    pub(super) infcx: &'a InferCtx<'a, 'tcx>,
    locals: RefCell<FxHashMap<ir::Id, LocalTy<'tcx>>>,
}

pub struct InheritedCtxBuilder<'tcx> {
    infcx: InferCtxBuilder<'tcx>,
}

#[derive(Debug, Clone, Copy)]
pub struct LocalTy<'tcx> {
    pub ty: Ty<'tcx>,
    pub mtbl: Mutability,
}

impl<'tcx> LocalTy<'tcx> {
    pub fn new(ty: Ty<'tcx>, mtbl: Mutability) -> Self {
        Self { ty, mtbl }
    }
}

impl<'tcx> InheritedCtxBuilder<'tcx> {
    pub fn enter<R>(&mut self, f: impl for<'a> FnOnce(InheritedCtx<'a, 'tcx>) -> R) -> R {
        self.infcx.enter(|infcx| f(InheritedCtx::new(&infcx)))
    }
}

impl<'a, 'tcx> InheritedCtx<'a, 'tcx> {
    pub fn new(infcx: &'a InferCtx<'a, 'tcx>) -> Self {
        Self { infcx, locals: Default::default() }
    }

    pub fn build(tcx: TyCtx<'tcx>, def_id: DefId) -> InheritedCtxBuilder<'tcx> {
        InheritedCtxBuilder { infcx: tcx.infer_ctx(def_id) }
    }

    /// top level entry point for typechecking a function item
    pub fn check_fn_item(
        &'a self,
        item: &ir::Item,
        sig: &ir::FnSig,
        generics: &ir::Generics,
        body: &ir::Body,
    ) -> FnCtx<'a, 'tcx> {
        let fn_ty = self.tcx.collected_ty(item.id.def);
        // don't instantiate anything and typeck the body using the param tys
        // don't know if this is a good idea
        let (_forall, ty) = fn_ty.expect_scheme();
        debug_assert_eq!(ty, TyConv::fn_sig_to_ty(self.infcx, sig));
        // check main function has the expected type fn() -> int
        if item.ident.symbol == symbol::MAIN && ty != self.types.main {
            self.emit_ty_err(item.span, TypeError::IncorrectMainType(ty));
        }
        self.check_fn(ty, body)
    }

    pub fn check_fn(&'a self, fn_ty: Ty<'tcx>, body: &ir::Body) -> FnCtx<'a, 'tcx> {
        let mut fcx = FnCtx::new(self, fn_ty);
        fcx.check_body(body);
        fcx
    }

    pub fn def_local(&self, id: ir::Id, ty: Ty<'tcx>, mtbl: Mutability) -> Ty<'tcx> {
        info!("deflocal {:?} : {}", id, ty);
        self.locals.borrow_mut().insert(id, LocalTy::new(ty, mtbl));
        ty
    }

    pub fn local_ty(&self, id: ir::Id) -> LocalTy<'tcx> {
        info!("lookup ty for local {:?}", id);
        self.locals.borrow().get(&id).cloned().expect("no entry for local variable")
    }
}

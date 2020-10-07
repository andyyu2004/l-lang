use crate::TyConv;
use ast::{Ident, Mutability};
use error::{LError, LResult};
use infer::TyCtxtInferExt;
use infer::{InferCtx, InferCtxBuilder};
use ir::{self, DefId, FnVisitor, ItemVisitor};
use lcore::ty::*;
use lcore::TyCtx;
use rustc_hash::FxHashMap;
use span::Span;
use std::cell::RefCell;
use std::io::Write;
use std::ops::Deref;
use tir::TirCtx;

macro halt_on_error($tcx:expr) {{
    if $tcx.sess.has_errors() {
        return Err(LError::ErrorReported);
    }
}}

pub fn typeck_fn<'tcx, R>(
    tcx: TyCtx<'tcx>,
    def_id: DefId,
    sig: &ir::FnSig<'tcx>,
    generics: &ir::Generics<'tcx>,
    body: &'tcx ir::Body<'tcx>,
    f: impl for<'a> FnOnce(TirCtx<'a, 'tcx>) -> R,
) -> LResult<R> {
    InheritedCtx::build(tcx, def_id).enter(|inherited| {
        let fcx = inherited.check_fn_item(def_id, sig, generics, body);
        // don't bother continuing if typeck failed
        // note that the failure to typeck could also come from resolution errors
        halt_on_error!(tcx);
        let tables = fcx.resolve_inference_variables(body);
        let lctx = TirCtx::new(&inherited, tables);
        Ok(f(lctx))
    })
}

pub struct FnCtx<'a, 'tcx> {
    inherited: &'a InheritedCtx<'a, 'tcx>,
    pub(super) param_tys: SubstsRef<'tcx>,
    pub(super) ret_ty: Ty<'tcx>,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn new(inherited: &'a InheritedCtx<'a, 'tcx>, fn_ty: Ty<'tcx>) -> Self {
        let (param_tys, ret_ty) = fn_ty.expect_fn();
        Self { inherited, param_tys, ret_ty }
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
        self.tcx.mk_substs(ir_tys.iter().map(|ty| self.ir_ty_to_ty(ty)))
    }

    pub fn lower_ty(&self, ir_ty: &ir::Ty) -> Ty<'tcx> {
        self.ir_ty_to_ty(ir_ty)
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
        def_id: DefId,
        sig: &ir::FnSig,
        _generics: &ir::Generics,
        body: &ir::Body,
    ) -> FnCtx<'a, 'tcx> {
        let fn_ty = self.tcx.collected_ty(def_id);
        // don't instantiate anything and typeck the body using the param tys
        // don't know if this is a good idea
        let (_forall, ty) = fn_ty.expect_scheme();
        debug_assert_eq!(ty, self.fn_sig_to_ty(sig));
        // TODO do this somewhere else
        // check main function has the expected type fn() -> int
        // if item.ident.symbol == symbol::MAIN && ty != self.types.main {
        // self.emit_ty_err(item.span, TypeError::IncorrectMainType(ty));
        // }
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
pub fn dump_mir<'tcx>(tcx: TyCtx<'tcx>, writer: &mut dyn Write) {
    MirDump { writer, tcx }.visit_ir(tcx.ir);
}

struct MirDump<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    writer: &'a mut dyn Write,
}

impl<'a, 'tcx> FnVisitor<'tcx> for MirDump<'a, 'tcx> {
    fn visit_fn(
        &mut self,
        def_id: DefId,
        _ident: Ident,
        sig: &'tcx ir::FnSig<'tcx>,
        generics: &'tcx ir::Generics<'tcx>,
        body: &'tcx ir::Body<'tcx>,
    ) {
        let _ = typeck_fn(self.tcx, def_id, sig, generics, body, |mut lctx| {
            let mir = lctx.build_mir(body);
            write!(self.writer, "\n{}", mir).unwrap();
        });
    }
}

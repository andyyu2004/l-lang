mod coerce;
mod expr;
mod item;
mod method_resolution;
mod pat;
mod path;
mod stmt;

use crate::TyConv;
use ir::{self, DefId};
use lc_ast::Mutability;
use lc_core::queries::Queries;
use lc_core::ty::*;
use lc_infer::{InferCtx, InferCtxBuilder, TyCtxtInferExt};
use lc_span::Span;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::ops::Deref;

pub fn provide(queries: &mut Queries) {
    item::provide(queries);
    *queries = Queries { typeck, ..*queries }
}

/// checks the bodies of item
fn typeck(tcx: TyCtx<'_>, def_id: DefId) -> &TypeckTables<'_> {
    let body = tcx.defs().body(def_id);
    InheritedCtx::build(tcx, def_id).enter(|inherited| {
        match tcx.sess.try_run(|| inherited.check_fn_item(def_id, body)) {
            // don't try and resolve inference variables if errors occured
            // as it makes a mess of diagnostics currently
            Ok(fcx) => fcx.resolve_inference_variables(body),
            Err(fcx) => tcx.alloc(fcx.tables.borrow().clone()),
        }
    })
}

pub struct FnCtx<'a, 'tcx> {
    pub(crate) sig: FnSig<'tcx>,
    inherited: &'a InheritedCtx<'a, 'tcx>,
    unsafe_ctx: bool,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn new(inherited: &'a InheritedCtx<'a, 'tcx>, sig: FnSig<'tcx>) -> Self {
        Self { inherited, sig, unsafe_ctx: false }
    }

    pub(crate) fn in_unsafe_ctx(&self) -> bool {
        self.unsafe_ctx
    }

    pub(crate) fn with_unsafe_ctx<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let old = self.unsafe_ctx;
        self.unsafe_ctx = true;
        let ret = f(self);
        self.unsafe_ctx = old;
        ret
    }
}

impl<'a, 'tcx> Deref for FnCtx<'a, 'tcx> {
    type Target = InheritedCtx<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        self.inherited
    }
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn lower_tys(&self, ir_tys: &[ir::Ty<'tcx>]) -> &'tcx [Ty<'tcx>] {
        self.tcx.mk_substs(ir_tys.iter().map(|ty| self.ir_ty_to_ty(ty)))
    }
}

impl<'a, 'tcx> TyConv<'tcx> for InferCtx<'a, 'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        self.tcx
    }

    fn infer_ty(&self, span: Span) -> Ty<'tcx> {
        self.new_infer_var(span)
    }

    fn allow_infer(&self) -> bool {
        true
    }
}

impl<'a, 'tcx> Deref for InheritedCtx<'a, 'tcx> {
    type Target = InferCtx<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        self.infcx
    }
}

/// context that is shared between functions
/// nested lambdas will have their own `FnCtx` but will share `Inherited` will outer lambdas as
/// well as the outermost fn item
pub struct InheritedCtx<'a, 'tcx> {
    pub(crate) infcx: &'a InferCtx<'a, 'tcx>,
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
    pub fn check_fn_item(&'a self, def_id: DefId, body: &ir::Body<'tcx>) -> FnCtx<'a, 'tcx> {
        let sig = self.tcx.fn_sig(def_id);
        // don't instantiate anything and typeck the body using the param tys
        self.check_fn(sig, body)
    }

    // common logic between closures and function items
    pub fn check_fn(&'a self, sig: FnSig<'tcx>, body: &ir::Body<'tcx>) -> FnCtx<'a, 'tcx> {
        let mut fcx = FnCtx::new(self, sig);
        fcx.check_body(body);
        fcx
    }

    pub fn def_local(&self, id: ir::Id, mtbl: Mutability, ty: Ty<'tcx>) -> Ty<'tcx> {
        debug!("deflocal {:?} : {}", id, ty);
        self.locals.borrow_mut().insert(id, LocalTy::new(ty, mtbl));
        ty
    }

    pub fn local_ty(&self, id: ir::Id) -> LocalTy<'tcx> {
        debug!("lookup ty for local {:?}", id);
        self.locals.borrow().get(&id).cloned().expect("no entry for local variable")
    }
}

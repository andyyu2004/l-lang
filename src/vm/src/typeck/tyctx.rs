use super::inference::InferCtxBuilder;
use super::{InferResult, TypeResult};
use crate::shared::{Arena, CtxInterners};
use crate::{
    ir, tir, ty::{Ty, TyKind}
};

#[derive(Copy, Clone, Deref)]
crate struct TyCtx<'tcx> {
    gcx: &'tcx GlobalCtx<'tcx>,
}

impl<'tcx> TyCtx<'tcx> {
    pub fn alloc_tir<T>(&self, tir: T) -> &'tcx T {
        self.gcx.interners.intern_tir(tir)
    }

    pub fn mk_ty(&self, ty: TyKind<'tcx>) -> Ty<'tcx> {
        self.gcx.interners.intern_ty(ty)
    }
}

crate struct GlobalCtx<'tcx> {
    interners: CtxInterners<'tcx>,
    pub types: CommonTypes<'tcx>,
}

impl<'tcx> GlobalCtx<'tcx> {
    pub fn new(arena: &'tcx Arena<'tcx>) -> Self {
        let interners = CtxInterners::new(arena);
        Self { types: CommonTypes::new(&interners), interners }
    }

    pub fn enter_tcx<R>(&'tcx self, f: impl FnOnce(TyCtx<'tcx>) -> R) -> R {
        let tcx = TyCtx { gcx: self };
        f(tcx)
    }
}

impl<'tcx> TyCtx<'tcx> {
    pub fn infer_ctx(self) -> InferCtxBuilder<'tcx> {
        InferCtxBuilder::new(self)
    }
}

impl<'tcx> TyCtx<'tcx> {
    pub fn type_expr(self, expr: &ir::Expr<'_>) -> InferResult<'tcx, &'tcx tir::Expr<'tcx>> {
        self.infer_ctx().enter(|infcx| infcx.infer_expr(expr))
    }
}

crate struct CommonTypes<'tcx> {
    pub unit: Ty<'tcx>,
    pub boolean: Ty<'tcx>,
    pub character: Ty<'tcx>,
    pub num: Ty<'tcx>,
}

impl<'tcx> CommonTypes<'tcx> {
    fn new(interners: &CtxInterners<'tcx>) -> CommonTypes<'tcx> {
        let mk = |ty| interners.intern_ty(ty);
        CommonTypes {
            unit: mk(TyKind::Unit),
            boolean: mk(TyKind::Bool),
            character: mk(TyKind::Char),
            num: mk(TyKind::Num),
        }
    }
}

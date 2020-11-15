use crate::queries::QueryCtx;
use crate::ty::*;
use crate::*;
use ast::Ident;
use index::IndexVec;
use ir::{DefId, FieldIdx, ParamIdx, VariantIdx};
use itertools::Itertools;
use resolve::Resolutions;
use session::Session;
use std::cell::Cell;
use std::ops::Deref;

/// typing context
#[derive(Copy, Clone)]
pub struct TyCtx<'tcx> {
    gcx: &'tcx GlobalCtx<'tcx>,
}

/// thread local storage for the TyCtx<'tcx>
pub mod tls {
    use super::*;

    // stores a pointer to the gcx
    thread_local! (static GCX: Cell<usize> = Cell::new(0));

    crate fn enter_tcx<'tcx, R>(gcx: &GlobalCtx<'tcx>, f: impl FnOnce(TyCtx<'tcx>) -> R) -> R {
        // permanently sets the pointer
        GCX.with(|ptr| ptr.set(gcx as *const _ as _));
        with_tcx(f)
    }

    pub fn with_tcx<'tcx, R>(f: impl FnOnce(TyCtx<'tcx>) -> R) -> R {
        let gcx_ptr = GCX.with(|gcx| gcx.get()) as *const GlobalCtx;
        let gcx = unsafe { &*gcx_ptr };
        let tcx = TyCtx { gcx };
        f(tcx)
    }
}

impl<'tcx> TyCtx<'tcx> {
    pub fn alloc<T>(self, t: T) -> &'tcx T
    where
        T: ArenaAllocatable<'tcx>,
    {
        // these types have their own typed arena
        debug_assert!(std::any::type_name_of_val(&t) != std::any::type_name::<TyKind>());
        debug_assert!(std::any::type_name_of_val(&t) != std::any::type_name::<Const>());
        self.interners.arena.alloc(t)
    }

    pub fn alloc_iter<I, T>(self, iter: I) -> &'tcx [T]
    where
        I: IntoIterator<Item = T>,
        T: ArenaAllocatable<'tcx>,
    {
        self.interners.arena.alloc_from_iter(iter)
    }

    pub fn mk_struct_ty(self, def_id: DefId, ident: Ident, variant: VariantTy) -> &'tcx AdtTy {
        self.mk_adt(def_id, AdtKind::Struct, ident, std::iter::once(variant).collect())
    }

    pub fn mk_enum_ty(
        self,
        def_id: DefId,
        ident: Ident,
        variants: IndexVec<VariantIdx, VariantTy>,
    ) -> &'tcx AdtTy {
        self.mk_adt(def_id, AdtKind::Enum, ident, variants)
    }

    pub fn mk_adt(
        self,
        def_id: DefId,
        kind: AdtKind,
        ident: Ident,
        variants: IndexVec<VariantIdx, VariantTy>,
    ) -> &'tcx AdtTy {
        debug_assert!(kind != AdtKind::Struct || variants.len() == 1);
        self.arena.alloc(AdtTy { ident, def_id, kind, variants })
    }

    pub fn mk_adt_ty(self, adt_ty: &'tcx AdtTy, substs: SubstsRef<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Adt(adt_ty, substs))
    }

    pub fn mk_opaque_ty(self, def: DefId, substs: SubstsRef<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Opaque(def, substs))
    }

    pub fn mk_array_ty(self, ty: Ty<'tcx>, n: usize) -> Ty<'tcx> {
        self.mk_ty(TyKind::Array(ty, n))
    }

    pub fn mk_ty(self, ty: TyKind<'tcx>) -> Ty<'tcx> {
        self.interners.intern_ty(ty)
    }

    pub fn mk_fn_sig(self, params: SubstsRef<'tcx>, ret: Ty<'tcx>) -> FnSig<'tcx> {
        FnSig { params, ret }
    }

    pub fn mk_fn_ptr(self, fn_sig: FnSig<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::FnPtr(fn_sig))
    }

    pub fn mk_ptr_ty(self, ty: Ty<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Ptr(ty))
    }

    pub fn mk_box_ty(self, ty: Ty<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Box(ty))
    }

    pub fn mk_ty_param(self, def_id: DefId, idx: ParamIdx, ident: Ident) -> Ty<'tcx> {
        self.mk_ty(TyKind::Param(ParamTy { def_id, idx, ident }))
    }

    /// returns the new type after applying a projection
    pub fn apply_projection(self, ty: Ty<'tcx>, proj: Projection<'tcx>) -> Ty<'tcx> {
        match proj {
            Projection::Deref => ty.deref_ty(),
            Projection::Field(_, ty) => ty,
            Projection::PointerCast(ty) => ty,
        }
    }

    pub fn mk_prim_ty(self, prim_ty: ir::PrimTy) -> Ty<'tcx> {
        match prim_ty {
            ir::PrimTy::Char => self.types.char,
            ir::PrimTy::Bool => self.types.bool,
            ir::PrimTy::Float => self.types.float,
            ir::PrimTy::Int => self.types.int,
        }
    }

    pub fn mk_tup(self, substs: SubstsRef<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Tuple(substs))
    }

    pub fn mk_tup_iter<I>(self, iter: I) -> Ty<'tcx>
    where
        I: Iterator<Item = Ty<'tcx>>,
    {
        self.mk_tup(self.mk_substs(iter))
    }

    pub fn concat_generics(
        self,
        g: &ty::Generics<'tcx>,
        h: &ty::Generics<'tcx>,
    ) -> &'tcx ty::Generics<'tcx> {
        let mut params = g.params.to_vec();
        params.extend(h.params);
        self.alloc(ty::Generics { params: self.alloc_iter(params) })
    }

    pub fn mk_ty_err(self) -> Ty<'tcx> {
        self.mk_ty(TyKind::Error)
    }

    pub fn project_field(
        self,
        lvalue: mir::Lvalue<'tcx>,
        field: FieldIdx,
        ty: Ty<'tcx>,
    ) -> mir::Lvalue<'tcx> {
        self.project_lvalue(lvalue, Projection::Field(field, ty))
    }

    pub fn project_deref(self, lvalue: mir::Lvalue<'tcx>) -> mir::Lvalue<'tcx> {
        self.project_lvalue(lvalue, Projection::Deref)
    }

    pub fn project_lvalue(
        self,
        mir::Lvalue { id, projs }: mir::Lvalue<'tcx>,
        proj: Projection<'tcx>,
    ) -> mir::Lvalue<'tcx> {
        let mut projs = projs.to_vec();
        projs.push(proj);
        mir::Lvalue { id, projs: self.intern_lvalue_projections(&projs) }
    }

    pub fn mk_substs<I>(self, iter: I) -> SubstsRef<'tcx>
    where
        I: IntoIterator<Item = Ty<'tcx>>,
    {
        self.intern_substs(&iter.into_iter().collect_vec())
    }

    pub fn intern_lvalue_projections(
        self,
        projs: &[Projection<'tcx>],
    ) -> &'tcx List<Projection<'tcx>> {
        if projs.is_empty() {
            List::empty()
        } else {
            self.interners.intern_lvalue_projections(projs)
        }
    }

    pub fn mk_const_int(self, i: i64) -> &'tcx Const<'tcx> {
        self.mk_const(ConstKind::Int(i))
    }

    pub fn mk_const_float(self, f: f64) -> &'tcx Const<'tcx> {
        self.mk_const(ConstKind::Float(f))
    }

    pub fn mk_const_bool(self, b: bool) -> &'tcx Const<'tcx> {
        self.mk_const(ConstKind::Bool(b))
    }

    pub fn mk_const_discr(self, discr: i16) -> &'tcx Const<'tcx> {
        self.mk_const(ConstKind::Discr(discr))
    }

    pub fn mk_const_unit(self) -> &'tcx Const<'tcx> {
        self.mk_const(ConstKind::Unit)
    }

    pub fn mk_const(self, kind: ConstKind) -> &'tcx Const<'tcx> {
        let ty = match kind {
            ConstKind::Float(_) => self.types.float,
            ConstKind::Int(_) => self.types.int,
            ConstKind::Discr(_) => self.types.discr,
            ConstKind::Bool(_) => self.types.bool,
            ConstKind::Unit => self.types.unit,
        };
        self.intern_const(Const { kind, ty })
    }

    pub fn intern_const(self, c: Const<'tcx>) -> &'tcx Const<'tcx> {
        self.interners.intern_const(c)
    }

    pub fn intern_substs(self, slice: &[Ty<'tcx>]) -> SubstsRef<'tcx> {
        if slice.is_empty() { Substs::empty() } else { self.interners.intern_substs(slice) }
    }
}

pub struct GlobalCtx<'tcx> {
    pub arena: &'tcx Arena<'tcx>,
    pub sess: &'tcx Session,
    pub ir: &'tcx ir::Ir<'tcx>,
    pub types: CommonTypes<'tcx>,
    pub resolutions: Resolutions<'tcx>,
    queries: QueryCtx<'tcx>,
    interners: CtxInterners<'tcx>,
}

impl<'tcx> GlobalCtx<'tcx> {
    pub fn new(
        ir: &'tcx ir::Ir<'tcx>,
        arena: &'tcx Arena<'tcx>,
        resolutions: Resolutions<'tcx>,
        sess: &'tcx Session,
        queries: QueryCtx<'tcx>,
    ) -> Self {
        let interners = CtxInterners::new(arena);
        let types = CommonTypes::new(&interners);
        Self { types, ir, arena, interners, resolutions, sess, queries }
    }

    pub fn enter_tcx<R>(&self, f: impl FnOnce(TyCtx<'tcx>) -> R) -> R {
        tls::enter_tcx(self, f)
    }
}

pub struct CommonTypes<'tcx> {
    pub char: Ty<'tcx>,
    pub bool: Ty<'tcx>,
    pub unit: Ty<'tcx>,
    pub discr: Ty<'tcx>,
    pub float: Ty<'tcx>,
    pub int: Ty<'tcx>,
    pub never: Ty<'tcx>,
    /// type of `main` must be `fn() -> int`
    pub main: Ty<'tcx>,
}

impl<'tcx> CommonTypes<'tcx> {
    fn new(interners: &CtxInterners<'tcx>) -> CommonTypes<'tcx> {
        let mk = |ty| interners.intern_ty(ty);
        let int = mk(TyKind::Int);
        CommonTypes {
            bool: mk(TyKind::Bool),
            char: mk(TyKind::Char),
            discr: mk(TyKind::Discr),
            never: mk(TyKind::Never),
            float: mk(TyKind::Float),
            main: mk(TyKind::FnPtr(FnSig { params: Substs::empty(), ret: int })),
            unit: mk(TyKind::Tuple(Substs::empty())),
            int,
        }
    }
}

impl<'tcx> Deref for TyCtx<'tcx> {
    type Target = GlobalCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.gcx
    }
}

impl<'tcx> Deref for GlobalCtx<'tcx> {
    type Target = QueryCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.queries
    }
}

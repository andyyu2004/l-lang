use super::inference::{FnCtx, InferCtx, InferCtxBuilder, InheritedCtx};
use crate::ast::{Ident, Mutability};
use crate::core::{Arena, CtxInterners};
use crate::driver::Session;
use crate::error::{LError, LResult, TypeResult};
use crate::ir::{self, DefId, Definitions, FieldIdx, ParamIdx, VariantIdx};
use crate::mir;
use crate::resolve::Resolutions;
use crate::span::Span;
use crate::tir::{self, TirCtx};
use crate::ty::{self, *};
use indexed_vec::{Idx, IndexVec};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Deref;

#[derive(Copy, Clone)]
pub struct TyCtx<'tcx> {
    gcx: &'tcx GlobalCtx<'tcx>,
}

impl<'tcx> Deref for TyCtx<'tcx> {
    type Target = GlobalCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.gcx
    }
}

impl<'tcx> TyCtx<'tcx> {
    pub fn alloc<T>(self, t: T) -> &'tcx T {
        // these types have their own typed arena
        debug_assert!(std::any::type_name_of_val(&t) != std::any::type_name::<TyKind>());
        debug_assert!(std::any::type_name_of_val(&t) != std::any::type_name::<Const>());
        self.interners.arena.alloc(t)
    }

    pub fn alloc_iter<I, T>(self, iter: I) -> &'tcx [T]
    where
        I: IntoIterator<Item = T>,
    {
        self.interners.arena.alloc_iter(iter)
    }

    pub fn alloc_tir<T>(self, tir: T) -> &'tcx T {
        self.interners.arena.alloc_tir(tir)
    }

    pub fn alloc_tir_iter<I, T>(self, iter: I) -> &'tcx [T]
    where
        I: IntoIterator<Item = T>,
    {
        self.interners.arena.alloc_tir_iter(iter)
    }

    pub fn mk_struct_ty(
        self,
        def_id: DefId,
        ident: Ident,
        variant: VariantTy<'tcx>,
    ) -> &'tcx AdtTy<'tcx> {
        self.mk_adt(def_id, AdtKind::Struct, ident, std::iter::once(variant).collect())
    }

    pub fn mk_enum_ty(
        self,
        def_id: DefId,
        ident: Ident,
        variants: IndexVec<VariantIdx, VariantTy<'tcx>>,
    ) -> &'tcx AdtTy<'tcx> {
        self.mk_adt(def_id, AdtKind::Enum, ident, variants)
    }

    pub fn mk_adt(
        self,
        def_id: DefId,
        kind: AdtKind,
        ident: Ident,
        variants: IndexVec<VariantIdx, VariantTy<'tcx>>,
    ) -> &'tcx AdtTy<'tcx> {
        self.arena.alloc(AdtTy { ident, def_id, kind, variants })
    }

    pub fn mk_adt_ty(self, adt_ty: &'tcx AdtTy<'tcx>, substs: SubstsRef<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Adt(adt_ty, substs))
    }

    pub fn mk_opaque_ty(self, def: DefId, substs: SubstsRef<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Opaque(def, substs))
    }

    pub fn mk_array_ty(self, ty: Ty<'tcx>, n: usize) -> Ty<'tcx> {
        self.mk_ty(TyKind::Array(ty, n))
    }

    pub fn mk_ty_scheme(self, forall: Forall<'tcx>, ty: Ty<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Scheme(forall, ty))
    }

    pub fn mk_ty(self, ty: TyKind<'tcx>) -> Ty<'tcx> {
        self.interners.intern_ty(ty)
    }

    pub fn mk_fn_ty(self, params: SubstsRef<'tcx>, ret: Ty<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Fn(params, ret))
    }

    pub fn mk_ptr_ty(self, m: Mutability, ty: Ty<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Ptr(m, ty))
    }

    pub fn mk_ty_param(self, def_id: DefId, idx: ParamIdx) -> Ty<'tcx> {
        self.mk_ty(TyKind::Param(ty::ParamTy { def_id, idx }))
    }

    pub fn mk_prim_ty(self, prim_ty: ir::PrimTy) -> Ty<'tcx> {
        match prim_ty {
            ir::PrimTy::Char => self.types.character,
            ir::PrimTy::Bool => self.types.boolean,
            ir::PrimTy::Float => self.types.float,
            ir::PrimTy::Int => self.types.int,
        }
    }

    /// finds the type of an item that was obtained during the collection phase
    pub fn collected_ty(self, def_id: DefId) -> Ty<'tcx> {
        self.collected_ty_opt(def_id)
            .unwrap_or_else(|| panic!("no collected type found for `{}`", def_id))
    }

    pub fn collected_ty_opt(self, def_id: DefId) -> Option<Ty<'tcx>> {
        self.collected_tys.borrow().get(&def_id).copied()
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

    pub fn mk_ty_err(self) -> Ty<'tcx> {
        self.mk_ty(TyKind::Error)
    }

    pub fn lvalue_project_field(
        self,
        lvalue: mir::Lvalue<'tcx>,
        field: FieldIdx,
        ty: Ty<'tcx>,
    ) -> mir::Lvalue<'tcx> {
        self.project_lvalue(lvalue, Projection::Field(field, ty))
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
        I: Iterator<Item = Ty<'tcx>>,
    {
        self.intern_substs(&iter.collect_vec())
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

    pub fn intern_const(self, c: Const<'tcx>) -> &'tcx Const<'tcx> {
        self.interners.intern_const(c)
    }

    pub fn intern_substs(self, slice: &[Ty<'tcx>]) -> SubstsRef<'tcx> {
        if slice.is_empty() { List::empty() } else { self.interners.intern_substs(slice) }
    }
}

pub struct GlobalCtx<'tcx> {
    pub arena: &'tcx Arena<'tcx>,
    pub types: CommonTypes<'tcx>,
    pub sess: &'tcx Session,
    pub ir: &'tcx ir::Prog<'tcx>,
    interners: CtxInterners<'tcx>,
    pub resolutions: &'tcx Resolutions,
    /// where the results of type collection are stored
    pub(super) collected_tys: RefCell<FxHashMap<DefId, Ty<'tcx>>>,
}

impl<'tcx> GlobalCtx<'tcx> {
    pub fn new(
        ir: &'tcx ir::Prog<'tcx>,
        arena: &'tcx Arena<'tcx>,
        resolutions: &'tcx Resolutions,
        sess: &'tcx Session,
    ) -> Self {
        let interners = CtxInterners::new(arena);
        let types = CommonTypes::new(&interners);
        Self { types, ir, arena, interners, resolutions, sess, collected_tys: Default::default() }
    }

    pub fn enter_tcx<R>(&'tcx self, f: impl FnOnce(TyCtx<'tcx>) -> R) -> R {
        let tcx = TyCtx { gcx: self };
        f(tcx)
    }
}

impl<'tcx> TyCtx<'tcx> {
    pub fn infer_ctx(self, def_id: DefId) -> InferCtxBuilder<'tcx> {
        InferCtxBuilder::new(self, def_id)
    }
}

impl<'tcx> TyConv<'tcx> for TyCtx<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx> {
        *self
    }

    fn infer_ty(&self, _span: Span) -> Ty<'tcx> {
        panic!("tyctx can't lower types with inference variables")
    }
}

macro halt_on_error($tcx:expr) {{
    if $tcx.sess.has_errors() {
        return Err(LError::ErrorReported);
    }
}}

impl<'tcx> TyCtx<'tcx> {
    /// top level entrace to typechecking
    /// does not check bodies, that is done when lowering to tir/mir
    pub fn run_typeck(self) {
        self.collect(self.ir);
    }

    /// runs all analyses on `self.ir`
    pub fn check(self) {
        // TODO abstract this pattern into a item or function visitor?
        // we can ignore the results as the driver will pick it up
        self.ir.items.iter().for_each(|(id, item)| match item.kind {
            ir::ItemKind::Fn(sig, generics, body) => {
                let _ = self.typeck_fn(id.def, sig, generics, body, |_| {});
            }
            _ => {}
        });
        self.ir.impl_items.values().for_each(|item| match item.kind {
            ir::ImplItemKind::Fn(sig, body) => {
                let _ = self.typeck_fn(item.id.def, sig, item.generics, body.unwrap(), |_| {});
            }
        })
    }

    /// constructs a TypeScheme from a type and its generics
    pub(super) fn generalize(self, generics: &ir::Generics, ty: Ty<'tcx>) -> Ty<'tcx> {
        let binders = self.alloc_iter(generics.params.iter().map(|p| p.index));
        let forall = Forall { binders };
        self.mk_ty_scheme(forall, ty)
    }

    pub fn typeck_fn<R>(
        self,
        def_id: DefId,
        sig: &ir::FnSig<'tcx>,
        generics: &ir::Generics<'tcx>,
        body: &'tcx ir::Body<'tcx>,
        f: impl for<'a> FnOnce(TirCtx<'a, 'tcx>) -> R,
    ) -> LResult<R> {
        InheritedCtx::build(self, def_id).enter(|inherited| {
            let fcx = inherited.check_fn_item(def_id, sig, generics, body);
            // don't bother continuing if typeck failed
            // note that the failure to typeck could also come from resolution errors
            halt_on_error!(self);
            let tables = fcx.resolve_inference_variables(body);
            let lctx = TirCtx::new(&inherited, tables);
            Ok(f(lctx))
        })
    }
}

/// debug
impl<'tcx> TyCtx<'tcx> {
    pub fn dump_collected_tys(self) {
        println!("type collection results");
        self.collected_tys.borrow().iter().for_each(|(k, v)| println!("{:?}: {}", k, v));
    }

    /// ir -> tir
    /// this isn't actually used in the compiler pipeline anymore, its mostly for testing and debugging
    /// some older tests rely on this
    fn build_tir_inner(self, prog: &ir::Prog<'tcx>) -> tir::Prog<'tcx> {
        let mut items = BTreeMap::new();

        for item in prog.items.values() {
            match item.kind {
                ir::ItemKind::Fn(sig, generics, body) => {
                    if let Ok(tir) = self.typeck_fn(item.id.def, sig, generics, body, |mut lctx| {
                        lctx.lower_item_tir(item)
                    }) {
                        items.insert(item.id, tir);
                    }
                }
                ir::ItemKind::Struct(..) => {}
                // note that no tir is generated for enum constructors
                // the constructor code is generated at mir level only
                ir::ItemKind::Enum(..) => {}
                ir::ItemKind::Impl { generics, trait_path, self_ty, impl_item_refs } =>
                    unimplemented!(),
            }
        }
        tir::Prog { items }
    }

    /// top level entrace to typechecking and lowering to tir
    pub fn build_tir(self) -> tir::Prog<'tcx> {
        self.build_tir_inner(self.ir)
    }
}

pub struct CommonTypes<'tcx> {
    pub unit: Ty<'tcx>,
    pub boolean: Ty<'tcx>,
    pub character: Ty<'tcx>,
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
            boolean: mk(TyKind::Bool),
            character: mk(TyKind::Char),
            never: mk(TyKind::Never),
            float: mk(TyKind::Float),
            main: mk(TyKind::Fn(List::empty(), int)),
            unit: mk(TyKind::Tuple(List::empty())),
            int,
        }
    }
}

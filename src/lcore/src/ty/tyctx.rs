use crate::ty::*;
use crate::*;
use ast::{Ident, Mutability};
use index::IndexVec;
use ir::CtorKind;
use ir::{DefId, FieldIdx, ParamIdx, VariantIdx};
use itertools::Itertools;
use resolve::Resolutions;
use rustc_hash::FxHashMap;
use session::Session;
use std::cell::RefCell;
use std::ops::Deref;

#[derive(Copy, Clone)]
pub struct TyCtx<'tcx> {
    gcx: &'tcx GlobalCtx<'tcx>,
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
        debug_assert!(kind != AdtKind::Struct || variants.len() == 1);
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

    pub fn mk_ty_scheme(self, forall: Generics<'tcx>, ty: Ty<'tcx>) -> Ty<'tcx> {
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
        self.mk_ty(TyKind::Param(ParamTy { def_id, idx }))
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
    interners: CtxInterners<'tcx>,
    pub arena: &'tcx CoreArenas<'tcx>,
    pub types: CommonTypes<'tcx>,
    pub sess: &'tcx Session,
    pub ir: &'tcx ir::IR<'tcx>,
    pub resolutions: &'tcx Resolutions,
    /// map from the `DefId` of a function to a list of
    /// all substitutions the function is instantiated with
    // TODO maybe there is a better data structure to use that won't require cloning
    pub(super) monomorphizations: RefCell<FxHashMap<DefId, Vec<SubstsRef<'tcx>>>>,
    /// where the results of type collection are stored
    pub(super) collected_tys: RefCell<FxHashMap<DefId, Ty<'tcx>>>,
}

impl<'tcx> GlobalCtx<'tcx> {
    pub fn new(
        ir: &'tcx ir::IR<'tcx>,
        arena: &'tcx CoreArenas<'tcx>,
        resolutions: &'tcx Resolutions,
        sess: &'tcx Session,
    ) -> Self {
        let interners = CtxInterners::new(arena);
        let types = CommonTypes::new(&interners);
        Self {
            types,
            ir,
            arena,
            interners,
            resolutions,
            sess,
            collected_tys: Default::default(),
            monomorphizations: Default::default(),
        }
    }

    pub fn enter_tcx<R>(&'tcx self, f: impl FnOnce(TyCtx<'tcx>) -> R) -> R {
        let tcx = TyCtx { gcx: self };
        f(tcx)
    }
}

impl<'tcx> TyCtx<'tcx> {
    // top level entrace to typechecking
    // does not check bodies, that is done when lowering to tir/mir
    // pub fn run_typeck(self) {
    //     self.collect(self.ir);
    // }

    // /// runs all analyses on `self.ir`
    // pub fn check(self) {
    //     // TODO abstract this pattern into a item or function visitor?
    //     // we can ignore the results as the driver will pick it up
    //     self.ir.items.iter().for_each(|(&id, item)| match item.kind {
    //         ir::ItemKind::Fn(sig, generics, body) => {
    //             let _ = self.typeck_fn(id, sig, generics, body, |_| {});
    //         }
    //         _ => {}
    //     });
    //     self.ir.impl_items.values().for_each(|item| match item.kind {
    //         ir::ImplItemKind::Fn(sig, body) => {
    //             let _ = self.typeck_fn(item.id.def, sig, item.generics, body, |_| {});
    //         }
    //     })
    // }

    // /// constructs a TypeScheme from a type and its generics
    // pub(super) fn generalize(self, generics: &ir::Generics, ty: Ty<'tcx>) -> Ty<'tcx> {
    //     let binders = self.alloc_iter(generics.params.iter().map(|p| p.index));
    //     let generics = self.lower_generics(generics);
    //     self.mk_ty_scheme(generics, ty)
    // }

    // pub fn build_mir(
    //     self,
    //     def_id: DefId,
    //     sig: &ir::FnSig<'tcx>,
    //     generics: &ir::Generics<'tcx>,
    //     body: &'tcx ir::Body<'tcx>,
    // ) -> LResult<&'tcx mir::Mir<'tcx>> {
    //     self.typeck_fn(def_id, sig, generics, body, |mut lctx| lctx.build_mir(body))
    // }

    // pub fn typeck_fn<R>(
    //     self,
    //     def_id: DefId,
    //     sig: &ir::FnSig<'tcx>,
    //     generics: &ir::Generics<'tcx>,
    //     body: &'tcx ir::Body<'tcx>,
    //     f: impl for<'a> FnOnce(TirCtx<'a, 'tcx>) -> R,
    // ) -> LResult<R> {
    //     InheritedCtx::build(self, def_id).enter(|inherited| {
    //         let fcx = inherited.check_fn_item(def_id, sig, generics, body);
    //         // don't bother continuing if typeck failed
    //         // note that the failure to typeck could also come from resolution errors
    //         halt_on_error!(self);
    //         let tables = fcx.resolve_inference_variables(body);
    //         let lctx = TirCtx::new(&inherited, tables);
    //         Ok(f(lctx))
    //     })
    // }
}

impl<'tcx> TyCtx<'tcx> {
    /// write collected ty to tcx map
    pub fn collect_ty(self, def: DefId, ty: Ty<'tcx>) -> Ty<'tcx> {
        self.collected_tys.borrow_mut().insert(def, ty);
        ty
    }

    pub fn variant_ty(
        self,
        ident: Ident,
        ctor: Option<DefId>,
        variant_kind: &ir::VariantKind<'tcx>,
    ) -> VariantTy<'tcx> {
        let mut seen = FxHashMap::default();
        // TODO check the number of generic params are correct
        let fields = self.arena.alloc_iter(variant_kind.fields().iter().map(|f| {
            if let Some(span) = seen.insert(f.ident, f.span) {
                self.sess.emit_error(span, TypeError::FieldAlreadyDeclared(f.ident, ident));
            }
            FieldTy { def_id: f.id.def, ident: f.ident, vis: f.vis, ir_ty: f.ty }
        }));
        VariantTy { ctor, ident, fields, ctor_kind: CtorKind::from(variant_kind) }
    }
}

/// debug
impl<'tcx> TyCtx<'tcx> {
    pub fn dump_collected_tys(self) {
        println!("type collection results");
        self.collected_tys.borrow().iter().for_each(|(k, v)| println!("{:?}: {}", k, v));
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

impl<'tcx> Deref for TyCtx<'tcx> {
    type Target = GlobalCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.gcx
    }
}
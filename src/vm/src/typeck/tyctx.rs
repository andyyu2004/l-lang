use super::inference::{FnCtx, InferCtx, InferCtxBuilder, Inherited};
use crate::core::{Arena, CtxInterners};
use crate::driver::Session;
use crate::error::TypeResult;
use crate::ir::{self, DefId, Definitions, ParamIdx};
use crate::span::Span;
use crate::tir::{self, IrLoweringCtx};
use crate::ty::{self, Forall, List, SubstsRef, Ty, TyConv, TyKind, VariantIdx};
use indexed_vec::{Idx, IndexVec};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Deref;
use ty::{AdtTy, VariantTy};

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

    pub fn mk_adt(
        self,
        def_id: DefId,
        variants: IndexVec<VariantIdx, VariantTy<'tcx>>,
    ) -> &'tcx AdtTy<'tcx> {
        self.arena.alloc(AdtTy { def_id, variants })
    }

    pub fn mk_adt_ty(self, adt_ty: &'tcx AdtTy<'tcx>, substs: SubstsRef<'tcx>) -> Ty<'tcx> {
        self.mk_ty(TyKind::Adt(adt_ty, self.intern_substs(&[])))
    }

    pub fn mk_empty_adt_ty(self, adt_ty: &'tcx AdtTy<'tcx>) -> Ty<'tcx> {
        self.mk_adt_ty(adt_ty, self.intern_substs(&[]))
    }

    pub fn mk_ty(self, ty: TyKind<'tcx>) -> Ty<'tcx> {
        self.interners.intern_ty(ty)
    }

    pub fn mk_ty_param(self, def_id: DefId, idx: ParamIdx) -> Ty<'tcx> {
        self.mk_ty(TyKind::Param(ty::ParamTy { def_id, idx }))
    }

    pub fn mk_prim_ty(self, prim_ty: ir::PrimTy) -> Ty<'tcx> {
        match prim_ty {
            ir::PrimTy::Char => self.types.character,
            ir::PrimTy::Bool => self.types.boolean,
            ir::PrimTy::Num => self.types.num,
        }
    }

    /// finds the type of an item that was obtained during the collection phase
    pub fn item_ty(self, def_id: DefId) -> Ty<'tcx> {
        self.item_tys.borrow().get(&def_id).expect("no type entry for item")
    }

    pub fn mk_tup<I>(self, iter: I) -> Ty<'tcx>
    where
        I: Iterator<Item = Ty<'tcx>>,
    {
        self.mk_ty(TyKind::Tuple(self.mk_substs(iter)))
    }

    pub fn mk_ty_err(self) -> Ty<'tcx> {
        self.mk_ty(TyKind::Error)
    }

    pub fn mk_substs<I>(self, iter: I) -> SubstsRef<'tcx>
    where
        I: Iterator<Item = Ty<'tcx>>,
    {
        self.intern_substs(&iter.collect_vec())
    }

    pub fn intern_substs(self, slice: &[Ty<'tcx>]) -> SubstsRef<'tcx> {
        if slice.is_empty() { List::empty() } else { self.interners.intern_substs(slice) }
    }
}

pub struct GlobalCtx<'tcx> {
    pub arena: &'tcx Arena<'tcx>,
    pub types: CommonTypes<'tcx>,
    pub session: &'tcx Session,
    interners: CtxInterners<'tcx>,
    defs: &'tcx Definitions,
    /// where the results of type collection are stored
    pub(super) item_tys: RefCell<FxHashMap<DefId, Ty<'tcx>>>,
}

impl<'tcx> GlobalCtx<'tcx> {
    pub fn new(arena: &'tcx Arena<'tcx>, defs: &'tcx Definitions, session: &'tcx Session) -> Self {
        let interners = CtxInterners::new(arena);
        Self {
            types: CommonTypes::new(&interners),
            arena,
            interners,
            defs,
            session,
            item_tys: Default::default(),
        }
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

impl<'tcx> TyCtx<'tcx> {
    /// top level entrace to typechecking
    pub fn check_prog(self, ir: &ir::Prog<'tcx>) -> tir::Prog<'tcx> {
        self.collect(ir);
        self.type_prog(ir)
    }

    /// constructs a TypeScheme from a type and its generics
    pub(super) fn generalize(self, generics: &ir::Generics, ty: Ty<'tcx>) -> Ty<'tcx> {
        let binders = self.alloc_iter(generics.params.iter().map(|p| p.index));
        let forall = Forall { binders };
        self.mk_ty(TyKind::Scheme(forall, ty))
    }

    /// ir -> tir (`type` the ir prog to form tir)
    fn type_prog(self, prog: &ir::Prog<'tcx>) -> tir::Prog<'tcx> {
        let mut items = BTreeMap::new();

        for (&id, item) in &prog.items {
            // only functions compile down to any runtime representation
            // data definitions such as structs and enums are purely a compile time concept
            match item.kind {
                ir::ItemKind::Fn(sig, generics, body) =>
                    Inherited::build(self, item.id.def).enter(|inherited| {
                        let fcx = inherited.check_fn_item(item, sig, generics, body);
                        let tables = fcx.resolve_inference_variables(body);
                        let mut lctx = IrLoweringCtx::new(&inherited, tables);
                        items.insert(id, lctx.lower_item(item));
                    }),
                ir::ItemKind::Struct(..) => {}
            };
        }
        tir::Prog { items }
    }
}

pub struct CommonTypes<'tcx> {
    pub unit: Ty<'tcx>,
    pub boolean: Ty<'tcx>,
    pub character: Ty<'tcx>,
    pub num: Ty<'tcx>,
    pub never: Ty<'tcx>,
    /// type of `main` must be `fn() -> number`
    pub main: Ty<'tcx>,
}

impl<'tcx> CommonTypes<'tcx> {
    fn new(interners: &CtxInterners<'tcx>) -> CommonTypes<'tcx> {
        let mk = |ty| interners.intern_ty(ty);
        let num = mk(TyKind::Num);
        CommonTypes {
            boolean: mk(TyKind::Bool),
            character: mk(TyKind::Char),
            never: mk(TyKind::Never),
            main: mk(TyKind::Fn(List::empty(), num)),
            unit: mk(TyKind::Tuple(List::empty())),
            num,
        }
    }
}

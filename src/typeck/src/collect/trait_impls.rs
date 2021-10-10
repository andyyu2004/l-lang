//! collect trait impls

// TODO straight copied from inherent_impls
// the current purpose of this is to validate trait impls (by running ir_ty_to_ty)

use crate::TyConv;
use ir::{DefId, Visitor};
use lcore::queries::Queries;
use lcore::ty::{self, TraitImpls, TyCtx};
use rustc_hash::FxHashMap;

pub(crate) fn provide(queries: &mut Queries) {
    *queries = Queries { trait_impls: |tcx, ()| trait_impls(tcx), trait_impls_of, ..*queries }
}

fn trait_impls_of<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> &'tcx [DefId] {
    tcx.trait_impls(()).trait_impls.get(&def_id).map_or(&[], |xs| &xs)
}

fn trait_impls<'tcx>(tcx: TyCtx<'tcx>) -> &'tcx TraitImpls {
    tcx.alloc(TraitImplCollector::new(tcx).collect())
}

/// collects inherent impls
/// i.e. implementation blocks on a type (not as part of a trait)
/// e.g. impl S { ...  }
/// the `DefId` of the `impl` item itself will be recorded
struct TraitImplCollector<'tcx> {
    tcx: TyCtx<'tcx>,
    trait_impls: FxHashMap<DefId, Vec<DefId>>,
}

impl<'tcx> ir::Visitor<'tcx> for TraitImplCollector<'tcx> {
    fn visit_item(&mut self, item: &'tcx ir::Item<'tcx>) {
        let tcx = self.tcx;
        let self_ty = match item.kind {
            ir::ItemKind::Impl { self_ty, trait_path: Some(_), .. } => tcx.ir_ty_to_ty(self_ty),
            _ => return,
        };

        // sanity check that these types are consistent
        debug_assert_eq!(self_ty, tcx.type_of(item.id.def));

        match self_ty.kind {
            ty::Box(..) => todo!(),
            ty::Array(..) => todo!(),
            ty::FnPtr(..) => todo!(),
            ty::Tuple(..) => todo!(),
            ty::Infer(..) => todo!(),
            ty::Ptr(..) => todo!(),
            ty::Param(..) => todo!(),
            ty::Opaque(..) => todo!(),
            ty::Adt(adt, _) => self.visit_def(adt.def_id, item.id.def),
            ty::Bool | ty::Discr | ty::Char | ty::Float | ty::Int => todo!(),
            ty::Never => todo!(),
            ty::Error => return,
        }
    }
}

impl<'tcx> TraitImplCollector<'tcx> {
    fn new(tcx: TyCtx<'tcx>) -> Self {
        Self { tcx, trait_impls: Default::default() }
    }

    fn collect(mut self) -> TraitImpls {
        self.visit_ir(self.tcx.ir);
        TraitImpls { trait_impls: self.trait_impls }
    }

    fn visit_def(&mut self, type_def_id: DefId, impl_def_id: DefId) {
        self.trait_impls.entry(type_def_id).or_insert_with(Default::default).push(impl_def_id);
    }
}

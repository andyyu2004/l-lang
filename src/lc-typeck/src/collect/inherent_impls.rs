//! collect inherent impls

use crate::TyConv;
use ir::{DefId, Visitor};
use lc_core::queries::Queries;
use lc_core::ty::{self, InherentImpls, TyCtx};
use rustc_hash::FxHashMap;

pub(crate) fn provide(queries: &mut Queries) {
    *queries =
        Queries { inherent_impls: |tcx, ()| inherent_impls(tcx), inherent_impls_of, ..*queries }
}

fn inherent_impls_of(tcx: TyCtx<'_>, def_id: DefId) -> &[DefId] {
    tcx.inherent_impls(()).inherent_impls.get(&def_id).map_or(&[], |xs| xs)
}

fn inherent_impls(tcx: TyCtx<'_>) -> &InherentImpls {
    tcx.alloc(InherentCollector::new(tcx).collect())
}

/// collects inherent impls
/// i.e. implementation blocks on a type (not as part of a trait)
/// e.g. impl S { ...  }
/// the `DefId` of the `impl` item itself will be recorded
struct InherentCollector<'tcx> {
    tcx: TyCtx<'tcx>,
    inherent_impls: FxHashMap<DefId, Vec<DefId>>,
}

impl<'tcx> ir::Visitor<'tcx> for InherentCollector<'tcx> {
    fn visit_item(&mut self, item: &'tcx ir::Item<'tcx>) {
        let tcx = self.tcx;
        let self_ty = match item.kind {
            // only visit inherent impls (i.e. there is no trait_path)
            ir::ItemKind::Impl { self_ty, trait_path: None, .. } => tcx.ir_ty_to_ty(self_ty),
            _ => return,
        };

        // sanity check that these types are consistent
        debug_assert_eq!(self_ty, tcx.type_of(item.id.def));

        match self_ty.kind {
            ty::Boxed(..) => todo!(),
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
            ty::Error => (),
        }
    }
}

impl<'tcx> InherentCollector<'tcx> {
    fn new(tcx: TyCtx<'tcx>) -> Self {
        Self { tcx, inherent_impls: Default::default() }
    }

    fn collect(mut self) -> InherentImpls {
        self.visit_ir(self.tcx.ir);
        InherentImpls { inherent_impls: self.inherent_impls }
    }

    fn visit_def(&mut self, type_def_id: DefId, impl_def_id: DefId) {
        self.inherent_impls.entry(type_def_id).or_insert_with(Default::default).push(impl_def_id);
    }
}

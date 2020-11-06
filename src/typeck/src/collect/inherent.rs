//! collect inherent impls

use crate::TyConv;
use ir::{DefId, Visitor};
use lcore::queries::Queries;
use lcore::ty::{self, InherentImpls, TyCtx};
use rustc_hash::FxHashMap;

crate fn provide(queries: &mut Queries) {
    // adding a unit parameter to make the parameterless function fit with the query structure
    // where they all have exactly one parameter
    *queries =
        Queries { inherent_impls: |tcx, ()| inherent_impls(tcx), inherent_impls_of, ..*queries }
}

fn inherent_impls_of<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> Vec<DefId> {
    tcx.inherent_impls(()).inherent_impls.get(&def_id).cloned().unwrap_or_else(Vec::new)
}

fn inherent_impls<'tcx>(tcx: TyCtx<'tcx>) -> InherentImpls {
    InherentCollector::new(tcx).collect()
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
        let self_ty = match item.kind {
            // only visit inherent impls (i.e. there is no trait_path)
            ir::ItemKind::Impl { self_ty, trait_path: None, .. } => self_ty,
            _ => return,
        };
        let tcx = self.tcx;
        let self_ty = tcx.ir_ty_to_ty(self_ty);
        let ty = tcx.type_of(item.id.def);

        // sanity check that these types are consistent
        assert_eq!(self_ty, ty);

        match self_ty.kind {
            ty::Array(_, _) => todo!(),
            ty::Fn(_, _) => todo!(),
            ty::Tuple(_) => todo!(),
            ty::Infer(_) => todo!(),
            ty::Ptr(_) => todo!(),
            ty::Param(_) => todo!(),
            ty::Scheme(_, _) => todo!(),
            ty::Box(_) => todo!(),
            ty::Opaque(_, _) => todo!(),
            ty::Bool | ty::Discr | ty::Char | ty::Float | ty::Int => todo!(),
            ty::Error | ty::Never => todo!(),
            ty::Adt(adt, _) => self.visit_def(adt.def_id, item.id.def),
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

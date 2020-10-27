//! resolves methods and associated functions relative to a type

use crate::FnCtx;
use ast::Ident;
use ir::{DefId, DefKind, ImplItemRef, Res};
use lcore::ty::{self, Ty};
use std::ops::Deref;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    crate fn resolve_type_relative_path(
        &mut self,
        ty: Ty<'tcx>,
        segment: &ir::PathSegment<'tcx>,
    ) -> Res {
        // TODO maybe require the generic args in segment later?
        ResolutionCtx::new(self, ty, segment.ident).resolve()
    }
}

#[derive(Debug)]
struct Candidate<'tcx> {
    def_id: DefId,
    def_kind: DefKind,
    _pd: std::marker::PhantomData<&'tcx ()>,
}

impl<'tcx> Candidate<'tcx> {
    pub fn new(def_id: DefId, def_kind: DefKind) -> Candidate<'tcx> {
        Self { def_id, def_kind, _pd: Default::default() }
    }
}

crate struct ResolutionCtx<'a, 'tcx> {
    fcx: &'a FnCtx<'a, 'tcx>,
    base_ty: Ty<'tcx>,
    ident: Ident,
    inherent_candidates: Vec<Candidate<'tcx>>,
}

trait InherentCandidates<'tcx> {
    fn inherent_candidates(&self, rcx: &mut ResolutionCtx);
}

impl<'a, 'tcx> ResolutionCtx<'a, 'tcx> {
    fn new(fcx: &'a FnCtx<'a, 'tcx>, base_ty: Ty<'tcx>, ident: Ident) -> Self {
        Self { fcx, base_ty, ident, inherent_candidates: Default::default() }
    }

    fn collect_inherent_candidates(&mut self) {
        let ty = self.base_ty;
        ty.inherent_candidates(self)
    }

    fn resolve(mut self) -> Res {
        self.collect_inherent_candidates();
        dbg!(&self.inherent_candidates);
        self.resolve_candidates()
    }

    /// chooses a single candidate from the possibilities and returns a resolution to it
    fn resolve_candidates(mut self) -> Res {
        if self.inherent_candidates.len() == 1 {
            let selected = self.inherent_candidates.pop().unwrap();
            Res::Def(selected.def_id, selected.def_kind)
        } else {
            todo!("more than one candidate")
        }
    }

    fn add_candidate(&mut self, candidate: Candidate<'tcx>) {
        self.inherent_candidates.push(candidate);
    }
}

impl<'tcx> InherentCandidates<'tcx> for Ty<'tcx> {
    fn inherent_candidates(&self, rcx: &mut ResolutionCtx) {
        match self.kind {
            ty::Adt(adt, _) => adt.def_id.inherent_candidates(rcx),
            _ => todo!(),
        }
    }
}

impl<'tcx> InherentCandidates<'tcx> for DefId {
    fn inherent_candidates(&self, rcx: &mut ResolutionCtx) {
        let inherent_impls = rcx.inherent_impls_of_def(*self);
        for impl_def_id in inherent_impls {
            let impl_block = rcx.ir.items[&impl_def_id];
            match impl_block.kind {
                ir::ItemKind::Impl { impl_item_refs, .. } =>
                    impl_item_refs.inherent_candidates(rcx),
                _ => unreachable!(),
            }
        }
    }
}

impl<'tcx, T> InherentCandidates<'tcx> for [T]
where
    T: InherentCandidates<'tcx>,
{
    fn inherent_candidates(&self, rcx: &mut ResolutionCtx) {
        self.iter().for_each(|t| t.inherent_candidates(rcx))
    }
}

impl<'tcx> InherentCandidates<'tcx> for ImplItemRef {
    fn inherent_candidates(&self, rcx: &mut ResolutionCtx) {
        let impl_item = rcx.ir.impl_items[&self.id];
        if impl_item.ident != rcx.ident {
            return;
        }
        let def_kind = impl_item.kind.def_kind();
        rcx.add_candidate(Candidate::new(self.id.0, def_kind));
    }
}

impl<'a, 'tcx> Deref for ResolutionCtx<'a, 'tcx> {
    type Target = FnCtx<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.fcx
    }
}

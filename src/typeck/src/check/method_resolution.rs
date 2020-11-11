//! resolves methods and associated functions relative to a type

use crate::FnCtx;
use ast::Ident;
use ir::{DefId, DefKind, ImplItemRef, Res};
use lcore::ty::{self, Subst, Ty};
use std::ops::Deref;

// some ideas for dealing with overlapping trait impls
// enum Specificity {
//     // impl<V> Trait for V {}
//     Generic,
//     // impl<V> Trait for V where V : Bounds {}
//     BoundedGeneric,
//     // impl Trait for T {}
//     Type,
// }

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    crate fn resolve_type_relative_path(
        &mut self,
        xpat: &dyn ir::ExprOrPat<'tcx>,
        self_ty: Ty<'tcx>,
        segment: &ir::PathSegment<'tcx>,
    ) -> Res {
        // TODO maybe require the generic args in segment later?
        MethodResolutionCtx::new(self, xpat, self_ty, segment.ident).resolve()
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

crate struct MethodResolutionCtx<'a, 'tcx> {
    fcx: &'a FnCtx<'a, 'tcx>,
    // the expression or pattern that we are resolving
    xpat: &'a dyn ir::ExprOrPat<'tcx>,
    self_ty: Ty<'tcx>,
    ident: Ident,
    inherent_candidates: Vec<Candidate<'tcx>>,
}

trait InherentCandidates<'tcx> {
    fn inherent_candidates(&self, rcx: &mut MethodResolutionCtx);
}

impl<'a, 'tcx> MethodResolutionCtx<'a, 'tcx> {
    fn new(
        fcx: &'a FnCtx<'a, 'tcx>,
        xpat: &'a dyn ir::ExprOrPat<'tcx>,
        self_ty: Ty<'tcx>,
        ident: Ident,
    ) -> Self {
        Self { fcx, self_ty, xpat, ident, inherent_candidates: Default::default() }
    }

    fn collect_inherent_candidates(&mut self) {
        let ty = self.self_ty;
        ty.inherent_candidates(self)
    }

    fn resolve(mut self) -> Res {
        self.collect_inherent_candidates();
        self.resolve_candidates()
    }

    /// chooses a single candidate from the possibilities and returns a resolution to it
    fn resolve_candidates(mut self) -> Res {
        if self.inherent_candidates.len() == 1 {
            let selected = self.inherent_candidates.pop().unwrap();
            Res::Def(selected.def_id, selected.def_kind)
        } else if self.inherent_candidates.len() < 1 {
            panic!("no candidates found")
        } else {
            todo!("more than one candidate")
        }
    }

    fn add_candidate(&mut self, candidate: Candidate<'tcx>) {
        self.inherent_candidates.push(candidate);
    }

    fn impl_self_ty(&self, impl_def_id: DefId) -> Ty<'tcx> {
        let self_ty = self.type_of(impl_def_id);
        let substs = self.fresh_substs_for_item(impl_def_id);
        self_ty.subst(self.tcx, substs)
    }
}

impl<'tcx> InherentCandidates<'tcx> for Ty<'tcx> {
    fn inherent_candidates(&self, rcx: &mut MethodResolutionCtx) {
        match self.kind {
            ty::Adt(adt, _) => adt.def_id.inherent_candidates(rcx),
            _ => todo!(),
        }
    }
}

impl<'tcx> InherentCandidates<'tcx> for DefId {
    fn inherent_candidates(&self, rcx: &mut MethodResolutionCtx) {
        let inherent_impls = rcx.inherent_impls_of(*self);

        for &impl_def_id in inherent_impls {
            let impl_block = rcx.ir.items[&impl_def_id];
            rcx.probe(|_| {
                let impl_self_ty = rcx.impl_self_ty(impl_def_id);

                // we only consider the impl if is "sufficiently general"
                // we consider the impl sufficiently general if
                // `impl_self_ty` can be unified to `rcx.self_ty`
                match rcx.at(rcx.xpat.span()).equate(rcx.self_ty, impl_self_ty) {
                    Ok(ty) => println!("{}", ty),
                    Err(_) => println!(
                        "impl_self_ty `{}` not sufficiently general for self_ty `{}`",
                        impl_self_ty, rcx.self_ty
                    ),
                };
            });

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
    fn inherent_candidates(&self, rcx: &mut MethodResolutionCtx) {
        self.iter().for_each(|t| t.inherent_candidates(rcx))
    }
}

impl<'tcx> InherentCandidates<'tcx> for ImplItemRef {
    fn inherent_candidates(&self, rcx: &mut MethodResolutionCtx) {
        let impl_item = rcx.ir.impl_items[&self.id];
        if impl_item.ident != rcx.ident {
            return;
        }
        let def_kind = impl_item.kind.def_kind();
        rcx.add_candidate(Candidate::new(self.id.0, def_kind));
    }
}

impl<'a, 'tcx> Deref for MethodResolutionCtx<'a, 'tcx> {
    type Target = FnCtx<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.fcx
    }
}

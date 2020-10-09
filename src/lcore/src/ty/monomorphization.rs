use crate::ty::{SubstsRef, TyCtx};
use ir::DefId;

impl<'tcx> TyCtx<'tcx> {
    pub fn add_monomorphization(self, def_id: DefId, substs: SubstsRef<'tcx>) {
        self.monomorphizations.borrow_mut().entry(def_id).or_default().push(substs);
    }

    pub fn monomorphizations_for(&self, def_id: DefId) -> Option<Vec<SubstsRef<'tcx>>> {
        self.monomorphizations.borrow().get(&def_id).cloned()
    }
}

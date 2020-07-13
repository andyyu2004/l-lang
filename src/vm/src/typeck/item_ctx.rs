use super::TyCtx;
use crate::ir::DefId;

crate struct ItemCtx<'tcx> {
    tcx: TyCtx<'tcx>,
    def_id: DefId,
}

impl<'tcx> ItemCtx<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, def_id: DefId) -> Self {
        Self { tcx, def_id }
    }
}

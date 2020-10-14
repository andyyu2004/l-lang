//! methods for manipulating ir on `TyCtx`
use crate::ty::TyCtx;
use ir::{DefId, DefNode};

impl<'tcx> TyCtx<'tcx> {
    pub fn impl_item(self, id: ir::ImplItemId) -> &'tcx ir::ImplItem<'tcx> {
        &self.ir.impl_items[&id]
    }

    pub fn defs(self) -> DefMap<'tcx> {
        DefMap { tcx: self }
    }
}

#[derive(Copy, Clone)]
pub struct DefMap<'ir> {
    tcx: TyCtx<'ir>,
}

impl<'ir> DefMap<'ir> {
    pub fn get(&self, def_id: DefId) -> DefNode<'ir> {
        self.tcx.resolutions.defs.get_def_node(def_id)
    }
}

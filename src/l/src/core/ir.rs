//! methods for manipulating ir on `TyCtx`

use crate::ir::{self, ImplItemId};
use crate::typeck::TyCtx;

impl<'tcx> TyCtx<'tcx> {
    pub fn impl_item(self, id: ImplItemId) -> &'tcx ir::ImplItem<'tcx> {
        &self.ir.impl_items[&id]
    }
}

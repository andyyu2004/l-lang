//! methods for manipulating ir on `TyCtx`
use crate::ty::TyCtx;
use ast::Ident;
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

    pub fn ident_of(&self, def_id: DefId) -> Ident {
        match self.get(def_id) {
            ir::DefNode::Item(item) => item.ident,
            ir::DefNode::ImplItem(_) => todo!(),
            ir::DefNode::ForeignItem(_) => todo!(),
            ir::DefNode::Ctor(variant) | ir::DefNode::Variant(variant) => {
                let adt_ident = self.ident_of(variant.adt_def_id);
                adt_ident.concat_as_path(variant.ident)
            }
        }
    }
}

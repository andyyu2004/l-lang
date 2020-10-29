//! methods for manipulating ir on `TyCtx`
use crate::ty::TyCtx;
use ast::Ident;
use ir::{DefId, DefKind, DefNode};
use span::Span;

impl<'tcx> TyCtx<'tcx> {
    pub fn impl_item(self, id: ir::ImplItemId) -> &'tcx ir::ImplItem<'tcx> {
        &self.ir.impl_items[&id]
    }

    pub fn defs(self) -> DefMap<'tcx> {
        DefMap { tcx: self }
    }
}

#[derive(Copy, Clone)]
pub struct DefMap<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> DefMap<'tcx> {
    pub fn get(&self, def_id: DefId) -> DefNode<'tcx> {
        self.tcx.resolutions.defs.get_def_node(def_id)
    }

    pub fn def_kind(&self, _def_id: DefId) -> DefKind {
        todo!()
    }

    pub fn span(&self, def_id: DefId) -> Span {
        match self.get(def_id) {
            DefNode::Item(item) => item.span,
            DefNode::ImplItem(_) => todo!(),
            DefNode::ForeignItem(item) => item.span,
            DefNode::Ctor(variant) | ir::DefNode::Variant(variant) => variant.span,
            DefNode::TyParam(param) => param.span,
        }
    }

    pub fn ident_of(&self, def_id: DefId) -> Ident {
        match self.get(def_id) {
            DefNode::TyParam(param) => param.ident,
            DefNode::Item(item) => item.ident,
            DefNode::ImplItem(impl_item) => impl_item.ident,
            DefNode::ForeignItem(foreign_item) => foreign_item.ident,
            DefNode::Ctor(variant) | ir::DefNode::Variant(variant) => {
                let adt_ident = self.ident_of(variant.adt_def_id);
                adt_ident.concat_as_path(variant.ident)
            }
        }
    }
}

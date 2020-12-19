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
            DefNode::ImplItem(item) => item.span,
            DefNode::TraitItem(item) => item.span,
            DefNode::ForeignItem(item) => item.span,
            DefNode::Ctor(variant) | DefNode::Variant(variant) => variant.span,
            DefNode::TyParam(param) => param.span,
            DefNode::Field(field) => field.span,
        }
    }

    pub fn body(&self, def_id: DefId) -> &'tcx ir::Body<'tcx> {
        match self.get(def_id) {
            DefNode::Item(item) => match item.kind {
                ir::ItemKind::Fn(.., body) => body,
                _ => panic!(),
            },
            DefNode::ImplItem(impl_item) => match impl_item.kind {
                ir::ImplItemKind::Fn(_, body) => body,
            },
            DefNode::TraitItem(trait_item) => match trait_item.kind {
                ir::TraitItemKind::Fn(_, body) => body.unwrap(),
            },
            DefNode::ForeignItem(..)
            | DefNode::Ctor(..)
            | DefNode::Variant(..)
            | DefNode::Field(..)
            | DefNode::TyParam(..) => panic!(),
        }
    }

    pub fn generics(&self, def_id: DefId) -> &'tcx ir::Generics<'tcx> {
        let node = self.get(def_id);
        match node {
            DefNode::Item(item) => match item.kind {
                ir::ItemKind::Fn(_, generics, _)
                | ir::ItemKind::Enum(generics, _)
                | ir::ItemKind::TypeAlias(generics, _)
                | ir::ItemKind::Struct(generics, _)
                | ir::ItemKind::Trait { generics, .. }
                | ir::ItemKind::Impl { generics, .. } => generics,
                ir::ItemKind::Mod(..) | ir::ItemKind::Use(..) | ir::ItemKind::Extern(..) =>
                    panic!(),
            },
            DefNode::ImplItem(impl_item) => impl_item.generics,
            DefNode::TraitItem(trait_item) => trait_item.generics,
            DefNode::ForeignItem(foreign_item) => match foreign_item.kind {
                ir::ForeignItemKind::Fn(_, generics) => generics,
            },
            // these inherit the generics of their parents
            DefNode::Ctor(variant) | DefNode::Variant(variant) => self.generics(variant.adt_def_id),
            DefNode::Field(..) | DefNode::TyParam(..) =>
                panic!("def node has no generics: {}", node.descr()),
        }
    }

    pub fn ident(&self, def_id: DefId) -> Ident {
        match self.get(def_id) {
            DefNode::TyParam(param) => param.ident,
            DefNode::Item(item) => item.ident,
            DefNode::ImplItem(impl_item) => impl_item.ident,
            DefNode::TraitItem(trait_item) => trait_item.ident,
            DefNode::ForeignItem(foreign_item) => foreign_item.ident,
            DefNode::Ctor(variant) | DefNode::Variant(variant) => {
                let adt_ident = self.ident(variant.adt_def_id);
                adt_ident.concat_as_path(variant.ident)
            }
            DefNode::Field(field) => field.ident,
        }
    }
}

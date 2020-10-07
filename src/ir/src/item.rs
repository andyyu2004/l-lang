use crate as ir;
use ast::{Ident, Visibility};
use span::Span;

#[derive(Debug)]
pub struct Item<'ir> {
    pub span: Span,
    pub id: ir::Id,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ir::ItemKind<'ir>,
}

impl<'ir> Item<'ir> {
    pub fn body(&self) -> &ir::Body<'ir> {
        match &self.kind {
            ItemKind::Fn(_, _, body) => body,
            _ => panic!(),
        }
    }
}

impl<'ir> Item<'ir> {
    pub fn generics(&self) -> &'ir ir::Generics<'ir> {
        match self.kind {
            ItemKind::Impl { generics: g, .. }
            | ItemKind::Fn(_, g, _)
            | ItemKind::Struct(g, _)
            | ItemKind::Enum(g, _) => g,
        }
    }
}

#[derive(Debug)]
pub enum ItemKind<'ir> {
    Fn(&'ir ir::FnSig<'ir>, &'ir ir::Generics<'ir>, &'ir ir::Body<'ir>),
    Struct(&'ir ir::Generics<'ir>, ir::VariantKind<'ir>),
    Enum(&'ir ir::Generics<'ir>, &'ir [ir::Variant<'ir>]),
    Impl {
        generics: &'ir ir::Generics<'ir>,
        trait_path: Option<&'ir ir::Path<'ir>>,
        self_ty: &'ir ir::Ty<'ir>,
        impl_item_refs: &'ir [ImplItemRef],
    },
}

#[derive(Debug)]
pub struct ImplItem<'ir> {
    pub id: ir::Id,
    pub ident: Ident,
    pub span: Span,
    pub vis: Visibility,
    pub generics: &'ir ir::Generics<'ir>,
    pub kind: ImplItemKind<'ir>,
}

#[derive(Debug)]
pub enum ImplItemKind<'ir> {
    Fn(&'ir ir::FnSig<'ir>, &'ir ir::Body<'ir>),
}

pub enum TraitItemKind<'ir> {
    Fn(&'ir ir::FnSig<'ir>, Option<&'ir ir::Body<'ir>>),
}

#[derive(Debug, Clone)]
pub struct ImplItemRef {
    pub id: ir::ImplItemId,
}

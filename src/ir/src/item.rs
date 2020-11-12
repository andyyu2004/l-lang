use crate::{self as ir, DefId, DefKind};
use ast::{Ident, Visibility};
use span::Span;

#[derive(Debug, Copy, Clone)]
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
    pub fn generics(&self) -> Option<&'ir ir::Generics<'ir>> {
        match self.kind {
            ItemKind::Impl { generics: g, .. }
            | ItemKind::Fn(_, g, _)
            | ItemKind::Struct(g, _)
            | ItemKind::TypeAlias(g, _)
            | ItemKind::Enum(g, _) => Some(g),
            ItemKind::Extern(_) => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ItemKind<'ir> {
    Fn(&'ir ir::FnSig<'ir>, &'ir ir::Generics<'ir>, &'ir ir::Body<'ir>),
    TypeAlias(&'ir ir::Generics<'ir>, &'ir ir::Ty<'ir>),
    Struct(&'ir ir::Generics<'ir>, ir::VariantKind<'ir>),
    Enum(&'ir ir::Generics<'ir>, &'ir [ir::Variant<'ir>]),
    Extern(&'ir [ir::ForeignItem<'ir>]),
    Impl {
        generics: &'ir ir::Generics<'ir>,
        trait_path: Option<&'ir ir::Path<'ir>>,
        self_ty: &'ir ir::Ty<'ir>,
        impl_item_refs: &'ir [ImplItemRef],
    },
}

#[derive(Debug, Clone)]
pub struct ForeignItem<'ir> {
    pub id: ir::Id,
    pub ident: Ident,
    pub span: Span,
    pub vis: Visibility,
    pub kind: ForeignItemKind<'ir>,
}

#[derive(Debug, Copy, Clone)]
pub enum ForeignItemKind<'ir> {
    Fn(&'ir ir::FnSig<'ir>, &'ir ir::Generics<'ir>),
}

#[derive(Debug)]
pub struct TraitItem<'ir> {
    pub generics: &'ir ir::Generics<'ir>,
}

#[derive(Debug, Copy, Clone)]
pub struct ImplItem<'ir> {
    pub id: ir::Id,
    pub impl_def_id: DefId,
    pub ident: Ident,
    pub span: Span,
    pub vis: Visibility,
    pub generics: &'ir ir::Generics<'ir>,
    pub kind: ImplItemKind<'ir>,
}

#[derive(Debug, Copy, Clone)]
pub enum ImplItemKind<'ir> {
    Fn(&'ir ir::FnSig<'ir>, &'ir ir::Body<'ir>),
}

impl<'ir> ImplItemKind<'ir> {
    pub fn def_kind(&self) -> DefKind {
        match self {
            ImplItemKind::Fn(..) => DefKind::AssocFn,
        }
    }
}

pub enum TraitItemKind<'ir> {
    Fn(&'ir ir::FnSig<'ir>, Option<&'ir ir::Body<'ir>>),
}

#[derive(Debug, Clone)]
pub struct ImplItemRef {
    pub id: ir::ImplItemId,
}

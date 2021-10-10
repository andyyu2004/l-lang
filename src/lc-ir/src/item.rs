use crate::{self as ir, DefId, DefKind};
use lc_ast::{Abi, Ident, Visibility};
use lc_span::{Span};

#[derive(Debug, Clone)]
pub struct Item<'ir> {
    pub id: ir::Id,
    pub span: Span,
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
            ItemKind::Impl { generics, .. }
            | ItemKind::Trait { generics, .. }
            | ItemKind::Fn(_, generics, _)
            | ItemKind::Struct(generics, _)
            | ItemKind::TypeAlias(generics, _)
            | ItemKind::Enum(generics, _) => Some(generics),
            ItemKind::Mod(..) | ItemKind::Use(..) | ItemKind::Extern(..) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ItemKind<'ir> {
    Fn(&'ir ir::FnSig<'ir>, &'ir ir::Generics<'ir>, &'ir ir::Body<'ir>),
    Use(&'ir ir::Path<'ir>),
    TypeAlias(&'ir ir::Generics<'ir>, &'ir ir::Ty<'ir>),
    Struct(&'ir ir::Generics<'ir>, ir::VariantKind<'ir>),
    Enum(&'ir ir::Generics<'ir>, &'ir [ir::Variant<'ir>]),
    Extern(Abi, &'ir [ir::ForeignItem<'ir>]),
    Mod(ir::Mod<'ir>),
    Trait {
        generics: &'ir ir::Generics<'ir>,
        trait_item_refs: &'ir [ir::TraitItemRef],
    },
    Impl {
        generics: &'ir ir::Generics<'ir>,
        trait_path: Option<&'ir ir::Path<'ir>>,
        self_ty: &'ir ir::Ty<'ir>,
        impl_item_refs: &'ir [ImplItemRef],
    },
}

#[derive(Debug, Copy, Clone)]
pub struct Mod<'ir> {
    pub span: Span,
    pub items: &'ir [ir::DefId],
}

#[derive(Debug, Clone)]
pub struct ForeignItem<'ir> {
    pub id: ir::Id,
    pub abi: Abi,
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
    pub id: ir::Id,
    pub ident: Ident,
    pub span: Span,
    pub vis: Visibility,
    pub generics: &'ir ir::Generics<'ir>,
    pub kind: TraitItemKind<'ir>,
}

#[derive(Debug, Clone)]
pub struct ImplItem<'ir> {
    pub id: ir::Id,
    pub impl_def_id: DefId,
    pub ident: Ident,
    pub span: Span,
    pub vis: Visibility,
    pub generics: &'ir ir::Generics<'ir>,
    pub kind: ImplItemKind<'ir>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum TraitItemKind<'ir> {
    Fn(&'ir ir::FnSig<'ir>, Option<&'ir ir::Body<'ir>>),
}

#[derive(Debug, Clone)]
pub struct TraitItemRef {
    pub id: ir::TraitItemId,
}

#[derive(Debug, Clone)]
pub struct ImplItemRef {
    pub id: ir::ImplItemId,
}

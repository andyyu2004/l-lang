use super::*;
use span::Span;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use util;

#[derive(Debug, PartialEq, Clone)]
pub struct Item<K = ItemKind> {
    pub span: Span,
    pub id: NodeId,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: K,
}

impl Item {
    pub fn generics(&self) -> Option<&Generics> {
        match &self.kind {
            ItemKind::Impl { generics: g, .. }
            | ItemKind::Fn(_, g, _)
            | ItemKind::Struct(g, _)
            | ItemKind::Enum(g, _) => Some(g),
            ItemKind::Extern(..) => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ItemKind {
    Fn(FnSig, Generics, Option<P<Expr>>),
    Enum(Generics, Vec<Variant>),
    Struct(Generics, VariantKind),
    Extern(Vec<P<ForeignItem>>),
    Impl { generics: Generics, trait_path: Option<Path>, self_ty: P<Ty>, items: Vec<P<AssocItem>> },
}

impl ItemKind {
    pub fn descr(&self) -> &str {
        match self {
            ItemKind::Fn(_, _, body) => match body {
                Some(_) => "function",
                None => "bodyless function",
            },
            ItemKind::Enum(..) => "enum",
            ItemKind::Struct(..) => "struct",
            ItemKind::Impl { .. } => "impl block",
            ItemKind::Extern(..) => "extern block",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssocItemKind {
    Fn(FnSig, Generics, Option<P<Expr>>),
}

impl TryFrom<ItemKind> for AssocItemKind {
    type Error = ItemKind;

    fn try_from(kind: ItemKind) -> Result<Self, Self::Error> {
        match kind {
            ItemKind::Fn(sig, generics, expr) => Ok(Self::Fn(sig, generics, expr)),
            ItemKind::Extern(..)
            | ItemKind::Enum(..)
            | ItemKind::Struct(..)
            | ItemKind::Impl { .. } => Err(kind),
        }
    }
}

pub type AssocItem = Item<AssocItemKind>;

pub type ForeignItem = Item<ForeignItemKind>;

#[derive(Debug, PartialEq, Clone)]
pub enum ForeignItemKind {
    Fn(FnSig, Generics),
}

impl TryFrom<ItemKind> for ForeignItemKind {
    type Error = ItemKind;

    fn try_from(kind: ItemKind) -> Result<Self, Self::Error> {
        match kind {
            ItemKind::Fn(sig, generics, expr) if expr.is_none() => Ok(Self::Fn(sig, generics)),
            _ => Err(kind),
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ItemKind::Fn(sig, _generics, body) => writeln!(
                f,
                "{} fn {}({}) -> {:?} {}",
                self.vis.node,
                self.ident,
                util::join(&sig.params, ", "),
                sig.ret_ty,
                body.as_ref().unwrap()
            ),
            ItemKind::Enum(_generics, _variants) => todo!(),
            ItemKind::Struct(_generics, _variant_kind) => todo!(),
            ItemKind::Impl { .. } => todo!(),
            ItemKind::Extern(_) => todo!(),
        }
    }
}

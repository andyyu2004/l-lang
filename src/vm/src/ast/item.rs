use super::*;
use crate::util;
use crate::{ir::DefKind, lexer::Symbol, span::Span};
use indexed_vec::Idx;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub struct Item<K = ItemKind> {
    pub span: Span,
    pub id: NodeId,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: K,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ItemKind {
    Fn(FnSig, Generics, Option<P<Expr>>),
    Enum(Generics, Vec<Variant>),
    Struct(Generics, VariantKind),
    Impl { generics: Generics, trait_path: Option<Path>, self_ty: P<Ty>, items: Vec<P<AssocItem>> },
}

impl ItemKind {
    pub fn descr(&self) -> &str {
        match self {
            ItemKind::Fn(..) => "function",
            ItemKind::Enum(..) => "enum",
            ItemKind::Struct(..) => "struct",
            ItemKind::Impl { .. } => "impl block",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssocItemKind {
    Fn(FnSig, Generics, Option<P<Expr>>),
}

impl AssocItemKind {
    pub fn def_kind(&self) -> DefKind {
        match self {
            Self::Fn(..) => DefKind::AssocFn,
        }
    }
}

impl TryFrom<ItemKind> for AssocItemKind {
    type Error = ItemKind;

    fn try_from(kind: ItemKind) -> Result<Self, Self::Error> {
        match kind {
            ItemKind::Fn(sig, generics, expr) => Ok(Self::Fn(sig, generics, expr)),
            ItemKind::Enum(..) | ItemKind::Struct(..) | ItemKind::Impl { .. } => Err(kind),
        }
    }
}

pub type AssocItem = Item<AssocItemKind>;

impl ItemKind {
    pub fn def_kind(&self) -> DefKind {
        match self {
            ItemKind::Fn(..) => DefKind::Fn,
            ItemKind::Enum(..) => DefKind::Enum,
            ItemKind::Struct(..) => DefKind::Struct,
            ItemKind::Impl { .. } => DefKind::Impl,
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ItemKind::Fn(sig, generics, body) => writeln!(
                f,
                "{} fn {}({}) -> {:?} {}",
                self.vis.node,
                self.ident,
                util::join(&sig.params, ", "),
                sig.ret_ty,
                body.as_ref().unwrap()
            ),
            ItemKind::Enum(generics, variants) => todo!(),
            ItemKind::Struct(generics, variant_kind) => todo!(),
            ItemKind::Impl { .. } => todo!(),
        }
    }
}

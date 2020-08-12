use super::*;
use crate::util;
use crate::{ir::DefKind, lexer::Symbol, span::Span};
use indexed_vec::Idx;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub struct Item {
    pub span: Span,
    pub id: NodeId,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ItemKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ItemKind {
    Fn(FnSig, Generics, Option<P<Expr>>),
    Enum(Generics, Vec<Variant>),
    Struct(Generics, VariantKind),
}

impl ItemKind {
    pub fn def_kind(&self) -> DefKind {
        match self {
            ItemKind::Fn(..) => DefKind::Fn,
            ItemKind::Enum(..) => DefKind::Enum,
            ItemKind::Struct(..) => DefKind::Struct,
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
                util::join(&sig.inputs, ", "),
                sig.output,
                body.as_ref().unwrap()
            ),
            ItemKind::Enum(generics, variants) => todo!(),
            ItemKind::Struct(generics, variant_kind) => todo!(),
        }
    }
}

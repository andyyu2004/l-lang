use super::{Block, FnSig, Generics, Ident, NodeId, Visibility, P};
use crate::span::Span;

#[derive(Debug, PartialEq, Clone)]
crate struct Item {
    pub span: Span,
    pub id: NodeId,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ItemKind,
}

#[derive(Debug, PartialEq, Clone)]
crate enum ItemKind {
    Fn(FnSig, Generics, Option<P<Block>>),
}

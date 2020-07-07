use super::{Block, FnSig, Generics, Ident, Visibility, P};
use crate::span::Span;

#[derive(Debug, PartialEq, Clone)]
crate struct Item {
    pub span: Span,
    pub ident: Ident,
    pub vis: Visibility,
    pub kind: ItemKind,
}

#[derive(Debug, PartialEq, Clone)]
crate enum ItemKind {
    Fn(FnSig, Generics, Option<P<Block>>),
}

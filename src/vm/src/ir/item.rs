use crate::ast::{Ident, Visibility};
use crate::ir;
use crate::{lexer::Symbol, span::Span};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Item<'ir> {
    pub span: Span,
    pub id: ir::Id,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ir::ItemKind<'ir>,
}

impl<'ir> Item<'ir> {
    pub fn generics(&self) -> &'ir ir::Generics<'ir> {
        match self.kind {
            ItemKind::Fn(_, g, _) | ItemKind::Struct(g, _) | ItemKind::Enum(g, _) => g,
        }
    }
}

#[derive(Debug)]
pub enum ItemKind<'ir> {
    Fn(&'ir ir::FnSig<'ir>, &'ir ir::Generics<'ir>, &'ir ir::Body<'ir>),
    Struct(&'ir ir::Generics<'ir>, ir::VariantKind<'ir>),
    Enum(&'ir ir::Generics<'ir>, &'ir [ir::Variant<'ir>]),
}

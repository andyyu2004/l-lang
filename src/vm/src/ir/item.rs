use super::*;
use crate::ast::{Ident, Visibility};
use crate::ir;
use crate::{lexer::Symbol, span::Span};
use std::marker::PhantomData;

#[derive(Debug)]
crate struct Item<'ir> {
    pub span: Span,
    pub id: ir::Id,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ir::ItemKind<'ir>,
}

#[derive(Debug)]
crate enum ItemKind<'ir> {
    Fn(&'ir FnSig<'ir>, &'ir Generics<'ir>, &'ir Body<'ir>),
}

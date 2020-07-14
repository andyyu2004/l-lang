use super::*;
use crate::ast::{Ident, Visibility};
use crate::ir;
use crate::{lexer::Symbol, span::Span, ty::Ty};
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;

#[derive(Debug)]
crate struct Item<'tcx> {
    pub span: Span,
    pub id: ir::Id,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ItemKind<'tcx>,
}

#[derive(Debug)]
crate enum ItemKind<'tcx> {
    Fn(Ty<'tcx>, &'tcx tir::Generics<'tcx>, &'tcx tir::Body<'tcx>),
}

impl<'tcx> Display for Item<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            ItemKind::Fn(sig, generics, body) => write!(f, "{}fn: {} {}", self.vis.node, sig, body),
        }
    }
}

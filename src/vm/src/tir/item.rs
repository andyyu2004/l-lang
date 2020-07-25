use super::*;
use crate::ast::{Ident, Visibility};
use crate::ir;
use crate::tir;
use crate::util;
use crate::{lexer::Symbol, span::Span, ty::Ty};
use std::fmt::{self, Display};
use std::marker::PhantomData;

#[derive(Debug)]
crate struct Item<'tcx> {
    pub span: Span,
    pub id: ir::Id,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: tir::ItemKind<'tcx>,
}

#[derive(Debug)]
crate enum ItemKind<'tcx> {
    Fn(&'tcx tir::FnSig<'tcx>, &'tcx tir::Generics<'tcx>, &'tcx tir::Body<'tcx>),
}

impl<'tcx> Display for Item<'tcx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        tir::Formatter::new(f).fmt_item(self)
    }
}

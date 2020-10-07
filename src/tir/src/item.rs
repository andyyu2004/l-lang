use super::*;
use ast::{Ident, Visibility};
use std::fmt::{self, Display};

#[derive(Debug)]
pub struct Item<'tcx> {
    pub span: Span,
    pub id: ir::Id,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: tir::ItemKind<'tcx>,
}

#[derive(Debug)]
pub enum ItemKind<'tcx> {
    Fn(Ty<'tcx>, tir::Generics<'tcx>, Box<tir::Body<'tcx>>),
}

impl<'tcx> Display for Item<'tcx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        tir::Formatter::new(f).fmt_item(self)
    }
}

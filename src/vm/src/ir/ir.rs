use crate::ir;
use crate::{ast::Ident, span::Span};
use std::marker::PhantomData;

crate struct Path<'ir> {
    pub span: Span,
    pub res: Res,
    pub segments: &'ir [PathSegment<'ir>],
}

crate enum Res {
    PrimTy(ir::PrimTy),
    Local(Ident),
}
crate struct PathSegment<'ir> {
    pub ident: Ident,
    pd: PhantomData<&'ir ()>,
}

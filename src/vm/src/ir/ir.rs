use crate::ir;
use crate::{ast::Ident, span::Span};
use ir::{Id, Res};
use std::marker::PhantomData;

#[derive(Debug)]
crate struct Path<'ir> {
    pub span: Span,
    pub res: Res,
    pub segments: &'ir [PathSegment<'ir>],
}

#[derive(Debug)]
crate struct PathSegment<'ir> {
    pub ident: Ident,
    pd: PhantomData<&'ir ()>,
}

#[derive(Debug)]
crate struct Block<'ir> {
    pub id: Id,
    pub span: Span,
    pub stmts: &'ir [ir::Stmt<'ir>],
    pub expr: Option<&'ir ir::Expr<'ir>>,
}

#[derive(Debug)]
crate struct Let<'ir> {
    pub id: Id,
    pub span: Span,
    pub pat: &'ir ir::Pattern<'ir>,
    pub ty: Option<&'ir ir::Ty<'ir>>,
    pub init: Option<&'ir ir::Expr<'ir>>,
}

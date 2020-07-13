use crate::ir;
use crate::{ast::Ident, span::Span};
use ir::{Expr, Id, Res};
use std::marker::PhantomData;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
crate struct BodyId(ir::Id);

#[derive(Debug)]
crate struct Generics<'ir> {
    /// just to make it not a ZST
    pub data: usize,
    pub pd: PhantomData<&'ir ()>,
}

#[derive(Debug)]
crate struct Body<'ir> {
    pub params: &'ir [ir::Param<'ir>],
    // the body is not necessarily a block (e.g. closures)
    pub expr: &'ir Expr<'ir>,
}

#[derive(Debug)]
crate struct FnSig<'ir> {
    // rest of parameter information is in `Body`
    pub inputs: &'ir [ir::Ty<'ir>],
    pub output: Option<&'ir ir::Ty<'ir>>,
}

#[derive(Debug)]
crate struct Path<'ir> {
    pub span: Span,
    pub res: Res,
    pub segments: &'ir [PathSegment<'ir>],
}

#[derive(Debug)]
crate struct Param<'ir> {
    // no type as it is in `FnSig`
    pub span: Span,
    pub id: ir::Id,
    pub pat: &'ir ir::Pattern<'ir>,
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

use crate::ast::Ident;
use crate::ir;
use crate::{lexer::Symbol, span::Span};
use ir::{Id, Res};
use std::fmt::{self, Display, Formatter};
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
    pub expr: &'ir ir::Expr<'ir>,
}

#[derive(Debug)]
crate enum MatchSource {
    Match,
    If,
}

#[derive(Debug)]
crate struct Arm<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub pat: &'ir ir::Pattern<'ir>,
    pub guard: Option<&'ir ir::Expr<'ir>>,
    pub body: &'ir ir::Expr<'ir>,
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
    pub id: ir::Id,
    pub pd: PhantomData<&'ir ()>,
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

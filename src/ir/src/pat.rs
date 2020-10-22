use crate as ir;
use ast::{Ident, Mutability};
use span::Span;

#[derive(Debug)]
pub struct Pattern<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub kind: ir::PatternKind<'ir>,
}

#[derive(Debug, Copy, Clone)]
pub enum PatternKind<'ir> {
    Box(&'ir ir::Pattern<'ir>),
    Lit(&'ir ir::Expr<'ir>),
    Binding(Ident, Option<&'ir ir::Pattern<'ir>>, Mutability),
    Tuple(&'ir [ir::Pattern<'ir>]),
    Variant(&'ir ir::Path<'ir>, &'ir [ir::Pattern<'ir>]),
    Path(&'ir ir::Path<'ir>),
    Wildcard,
}

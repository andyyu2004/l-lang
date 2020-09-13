use crate::ast::{Ident, Mutability};
use crate::ir;
use crate::span::Span;

#[derive(Debug)]
pub struct Pattern<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub kind: ir::PatternKind<'ir>,
}

#[derive(Debug, Copy, Clone)]
pub enum PatternKind<'ir> {
    Wildcard,
    Lit(&'ir ir::Expr<'ir>),
    Binding(Ident, Option<&'ir ir::Pattern<'ir>>, Mutability),
    Tuple(&'ir [ir::Pattern<'ir>]),
}

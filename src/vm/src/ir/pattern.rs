use crate::ast::Ident;
use crate::ir;
use crate::span::Span;

#[derive(Debug)]
pub struct Pattern<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub kind: ir::PatternKind<'ir>,
}

#[derive(Debug)]
pub enum PatternKind<'ir> {
    Wildcard,
    Lit(&'ir ir::Expr<'ir>),
    Binding(Ident, Option<&'ir ir::Pattern<'ir>>),
    Tuple(&'ir [ir::Pattern<'ir>]),
}

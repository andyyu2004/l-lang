use super::{Expr, Id};
use crate::span::Span;

#[derive(Debug)]
crate struct Pattern<'ir> {
    pub id: Id,
    pub span: Span,
    pub kind: PatternKind<'ir>,
}

#[derive(Debug)]
crate enum PatternKind<'ir> {
    Wildcard,
    Todo(Expr<'ir>),
}

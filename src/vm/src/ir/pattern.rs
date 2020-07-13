use crate::ir;
use crate::{ast::Ident, span::Span};

#[derive(Debug)]
crate struct Pattern<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub kind: ir::PatternKind<'ir>,
}

#[derive(Debug)]
crate enum PatternKind<'ir> {
    Wildcard,
    Binding(Ident, Option<&'ir ir::Pattern<'ir>>),
}

use super::{Ident, NodeId, P};
use crate::span::Span;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
crate struct Pattern {
    pub span: Span,
    pub id: NodeId,
    pub kind: PatternKind,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
crate enum PatternKind {
    /// _
    Wildcard,
    /// ident ( @ <subpattern> )?
    Ident(Ident, Option<P<Pattern>>),
    Paren(P<Pattern>),
}

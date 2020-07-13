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

impl Pattern {
    /// call f on every subpattern
    /// if `f` returns false, stop recursing deeper
    pub fn walk(&self, f: &mut impl FnMut(&Self) -> bool) {
        if !f(self) {
            return;
        }
        match &self.kind {
            PatternKind::Ident(_, Some(p)) => p.walk(f),
            PatternKind::Paren(p) => p.walk(f),
            PatternKind::Wildcard | PatternKind::Ident(..) => {}
        }
    }
}

use super::{Expr, Ident, NodeId, Path, P};
use span::Span;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub struct Pattern {
    pub span: Span,
    pub id: NodeId,
    pub kind: PatternKind,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Copy)]
pub enum Mutability {
    Mut,
    Imm,
}

impl Display for Mutability {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Mutability::Mut => write!(f, "mut "),
            Mutability::Imm => Ok(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PatternKind {
    /// _
    Wildcard,
    /// ident ( @ <subpattern> )?
    Ident(Ident, Option<P<Pattern>>, Mutability),
    Paren(P<Pattern>),
    Tuple(Vec<P<Pattern>>),
    Lit(P<Expr>),
    /// Adt::Variant(..)
    /// also matches tuple structs
    Variant(Path, Vec<P<Pattern>>),
    /// can refer to unit variants and structs
    Path(Path),
    // Struct(Path, FieldPat),
}

use super::{Path, P};
use crate::span::Span;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
crate struct Ty {
    pub span: Span,
    pub kind: TyKind,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
crate enum TyKind {
    Array(P<Ty>),
    Tuple(Vec<P<Ty>>),
    Paren(P<Ty>),
    Path(Path),
}

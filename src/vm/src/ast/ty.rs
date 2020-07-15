use super::{NodeId, Path, P};
use crate::span::Span;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
crate struct Ty {
    pub span: Span,
    pub id: NodeId,
    pub kind: TyKind,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
crate enum TyKind {
    Array(P<Ty>),
    Tuple(Vec<P<Ty>>),
    Paren(P<Ty>),
    Path(Path),
    /// fn(<ty>...) (-> <ty>)?
    Fn(Vec<P<Ty>>, Option<P<Ty>>),
    Infer,
}

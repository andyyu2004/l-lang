use crate::span::Span;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Ty {
    pub span: Span,
    pub kind: TyKind,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum TyKind {}

use crate::span::Span;

pub struct Ty {
    pub span: Span,
    pub kind: TyKind,
}

pub enum TyKind {}

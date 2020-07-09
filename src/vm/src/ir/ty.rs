use super::{ir, Id};
use crate::span::Span;

#[derive(Debug)]
crate enum PrimTy {
    Char,
    Bool,
    Uint,
    Int,
    Float,
}

#[derive(Debug)]
crate struct Ty<'ir> {
    pub id: Id,
    pub span: Span,
    pub kind: TyKind<'ir>,
}

#[derive(Debug)]
crate enum TyKind<'ir> {
    Array(&'ir Ty<'ir>),
}

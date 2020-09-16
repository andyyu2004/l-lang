use crate::ast::Mutability;
use crate::ir;
use crate::span::Span;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PrimTy {
    Char,
    Bool,
    Float,
    Int,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct Ty<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub kind: ir::TyKind<'ir>,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum TyKind<'ir> {
    Path(&'ir ir::Path<'ir>),
    Array(&'ir ir::Ty<'ir>),
    Tuple(&'ir [ir::Ty<'ir>]),
    Ptr(Mutability, &'ir ir::Ty<'ir>),
    Fn(&'ir [ir::Ty<'ir>], Option<&'ir ir::Ty<'ir>>),
    Infer,
}

use crate as ir;
use ast::Mutability;
use span::Span;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PrimTy {
    Char,
    Bool,
    Float,
    Int,
}

#[derive(Debug)]
pub struct Ty<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub kind: ir::TyKind<'ir>,
}

#[derive(Debug)]
pub enum TyKind<'ir> {
    Box(Mutability, &'ir ir::Ty<'ir>),
    Fn(&'ir [ir::Ty<'ir>], Option<&'ir ir::Ty<'ir>>),
    Path(&'ir ir::Path<'ir>),
    Array(&'ir ir::Ty<'ir>),
    Tuple(&'ir [ir::Ty<'ir>]),
    Ptr(&'ir ir::Ty<'ir>),
    Infer,
    Err,
}

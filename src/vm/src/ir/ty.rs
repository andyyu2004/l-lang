use crate::ir;
use crate::{span::Span, ty::List};

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
    Path(&'ir ir::Path<'ir>),
    Array(&'ir ir::Ty<'ir>),
    Tuple(&'ir [ir::Ty<'ir>]),
    Fn(&'ir [ir::Ty<'ir>], Option<&'ir ir::Ty<'ir>>),
    Infer,
}

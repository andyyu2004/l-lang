use crate::ir;
use crate::{span::Span, typeck::List};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
crate enum PrimTy {
    Char,
    Bool,
    Num,
}

#[derive(Debug)]
crate struct Ty<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub kind: ir::TyKind<'ir>,
}

#[derive(Debug)]
crate enum TyKind<'ir> {
    Path(&'ir ir::Path<'ir>),
    Array(&'ir ir::Ty<'ir>),
}

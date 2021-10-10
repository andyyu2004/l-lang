use crate::{self as ir, QPath};
use lc_span::Span;
use std::fmt::{self, Display, Formatter};

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
    Box(&'ir ir::Ty<'ir>),
    Fn(&'ir [ir::Ty<'ir>], Option<&'ir ir::Ty<'ir>>),
    Path(&'ir QPath<'ir>),
    Array(&'ir ir::Ty<'ir>),
    Tuple(&'ir [ir::Ty<'ir>]),
    Ptr(&'ir ir::Ty<'ir>),
    Infer,
    Err,
}

impl<'ir> Display for Ty<'ir> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            ir::TyKind::Box(ty) => write!(f, "box {}", ty),
            ir::TyKind::Path(qpath) => write!(f, "{}", qpath),
            _ => todo!(),
        }
    }
}

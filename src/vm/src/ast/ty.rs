use super::{NodeId, Path, P};
use crate::span::Span;
use crate::util;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Ty {
    pub span: Span,
    pub id: NodeId,
    pub kind: TyKind,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum TyKind {
    Array(P<Ty>),
    Tuple(Vec<P<Ty>>),
    Paren(P<Ty>),
    Path(Path),
    /// fn(<ty>...) (-> <ty>)?
    Fn(Vec<P<Ty>>, Option<P<Ty>>),
    Infer,
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            _ => todo!(),
        }
    }
}

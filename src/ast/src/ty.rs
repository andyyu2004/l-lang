use super::{NodeId, Path, P};
use span::Span;
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
    /// (<ty>)
    Paren(P<Ty>),
    Path(Path),
    /// &<ty>
    Box(P<Ty>),
    /// fn(<ty>...) (-> <ty>)?
    Fn(Vec<P<Ty>>, Option<P<Ty>>),
    /// *<ty>
    Ptr(P<Ty>),
    /// type of a self parameter
    ImplicitSelf,
    /// _
    Infer,
    Err,
}

impl Display for Ty {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            _ => todo!(),
        }
    }
}

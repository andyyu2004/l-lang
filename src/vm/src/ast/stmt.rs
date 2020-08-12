use super::{Expr, Item, NodeId, Pattern, Ty, P};
use crate::span::Span;

#[derive(Debug, PartialEq, Clone)]
pub struct Stmt {
    pub span: Span,
    pub id: NodeId,
    pub kind: StmtKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StmtKind {
    /// let binding
    Let(P<Let>),
    /// no trailing semicolon
    Expr(P<Expr>),
    /// expression stmt (with trailing semicolon)
    Semi(P<Expr>),
    /// return stmt
    Ret(Option<P<Expr>>),
}

/// let <pat>:<ty> = <init>;
#[derive(Debug, PartialEq, Clone)]
pub struct Let {
    pub id: NodeId,
    pub span: Span,
    pub pat: P<Pattern>,
    pub ty: Option<P<Ty>>,
    pub init: Option<P<Expr>>,
}

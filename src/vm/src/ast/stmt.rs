use super::{Expr, Item, Pattern, Ty, P};
use crate::span::Span;

#[derive(Debug, PartialEq, Clone)]
crate struct Stmt {
    pub span: Span,
    pub kind: StmtKind,
}

#[derive(Debug, PartialEq, Clone)]
crate enum StmtKind {
    /// let binding
    Let(P<Let>),
    /// item declaration
    Item(P<Item>),
    /// no trailing semicoon
    Expr(P<Expr>),
    /// expression statement (with trailing semicolon)
    Semi(P<Expr>),
    Empty,
}

/// let <pat>:<ty> = <init>;
#[derive(Debug, PartialEq, Clone)]
crate struct Let {
    pub span: Span,
    pub pat: P<Pattern>,
    pub ty: Option<P<Ty>>,
    pub init: Option<P<Expr>>,
}

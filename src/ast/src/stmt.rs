use super::{Expr, NodeId, Pattern, Ty, P};
use span::Span;

#[derive(Debug, PartialEq, Clone)]
pub struct Stmt {
    pub span: Span,
    pub id: NodeId,
    pub kind: StmtKind,
}

impl Stmt {
    /// if the stmt is diverging e.g. return, break, continue,
    /// then change `Semi` to `Expr` for easier typechecking
    pub fn upgrade_diverging_to_expr(self) -> Self {
        return self;
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StmtKind {
    /// let binding
    Let(P<Let>),
    /// no trailing semicolon
    Expr(P<Expr>),
    /// expression stmt (with trailing semicolon)
    Semi(P<Expr>),
}

/// let rec? <pat>:<ty> = <init>;
#[derive(Debug, PartialEq, Clone)]
pub struct Let {
    pub id: NodeId,
    pub span: Span,
    pub pat: P<Pattern>,
    pub ty: Option<P<Ty>>,
    pub init: Option<P<Expr>>,
}

use super::{Expr, Id, Let};
use crate::span::Span;

#[derive(Debug)]
crate struct Stmt<'ir> {
    pub id: Id,
    pub span: Span,
    pub kind: StmtKind<'ir>,
}

#[derive(Debug)]
crate enum StmtKind<'ir> {
    Let(&'ir Let<'ir>),
    Expr(&'ir Expr<'ir>),
    Semi(&'ir Expr<'ir>),
}

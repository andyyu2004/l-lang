use crate as ir;
use span::Span;

#[derive(Debug)]
pub struct Stmt<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub kind: ir::StmtKind<'ir>,
}

#[derive(Debug)]
pub enum StmtKind<'ir> {
    Let(&'ir ir::Let<'ir>),
    Expr(&'ir ir::Expr<'ir>),
    Semi(&'ir ir::Expr<'ir>),
}

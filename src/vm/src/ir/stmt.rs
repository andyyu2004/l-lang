use crate::ir;
use crate::span::Span;

#[derive(Debug)]
crate struct Stmt<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub kind: ir::StmtKind<'ir>,
}

#[derive(Debug)]
crate enum StmtKind<'ir> {
    Let(&'ir ir::Let<'ir>),
    Expr(&'ir ir::Expr<'ir>),
    Semi(&'ir ir::Expr<'ir>),
    Ret(Option<&'ir ir::Expr<'ir>>),
}

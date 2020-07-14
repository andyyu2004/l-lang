use super::Block;
use crate::ast;
use crate::ir;
use crate::span::Span;

#[derive(Debug)]
crate struct Expr<'ir> {
    pub span: Span,
    pub id: ir::Id,
    pub kind: ir::ExprKind<'ir>,
}

impl<'ir> From<&'ir ir::Block<'ir>> for Expr<'ir> {
    fn from(block: &'ir ir::Block<'ir>) -> Self {
        let kind = ExprKind::Block(block);
        Expr { span: block.span, id: block.id, kind }
    }
}

#[derive(Debug)]
crate enum ExprKind<'ir> {
    Lit(ast::Lit),
    Bin(ast::BinOp, &'ir ir::Expr<'ir>, &'ir ir::Expr<'ir>),
    Unary(ast::UnaryOp, &'ir ir::Expr<'ir>),
    Block(&'ir ir::Block<'ir>),
    Path(&'ir ir::Path<'ir>),
}

use super::Block;
use crate::ast;
use crate::ir;
use crate::span::Span;
use ast::Ident;
use std::fmt::Display;

#[derive(Debug)]
pub struct Expr<'ir> {
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

impl<'ir> Expr<'ir> {
    pub fn lvalue(&self) -> Option<ir::Lvalue> {
        match self.kind {
            ExprKind::Path(p) => match p.res {
                ir::Res::Local(id) => Some(ir::Lvalue::Local(id)),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ExprKind<'ir> {
    Lit(ast::Lit),
    Bin(ast::BinOp, &'ir ir::Expr<'ir>, &'ir ir::Expr<'ir>),
    Unary(ast::UnaryOp, &'ir ir::Expr<'ir>),
    Ret(Option<&'ir ir::Expr<'ir>>),
    Block(&'ir ir::Block<'ir>),
    Path(&'ir ir::Path<'ir>),
    Tuple(&'ir [ir::Expr<'ir>]),
    Closure(&'ir ir::FnSig<'ir>, &'ir ir::Body<'ir>),
    Assign(&'ir ir::Expr<'ir>, &'ir ir::Expr<'ir>),
    Call(&'ir ir::Expr<'ir>, &'ir [ir::Expr<'ir>]),
    Match(&'ir ir::Expr<'ir>, &'ir [ir::Arm<'ir>], ir::MatchSource),
    Struct(&'ir ir::Path<'ir>, &'ir [ir::Field<'ir>]),
    /// named field access `foo.x` or `tuple.1`
    Field(&'ir ir::Expr<'ir>, Ident),
}

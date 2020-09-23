use super::Block;
use crate::ast::{self, Ident, UnaryOp};
use crate::ir;
use crate::span::Span;
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
    pub fn is_lvalue(&self) -> bool {
        match self.kind {
            ExprKind::Path(p) => match p.res {
                ir::Res::Local(id) => true,
                ir::Res::Def(_, def_kind) => match def_kind {
                    ir::DefKind::Fn => false,
                    ir::DefKind::Enum => false,
                    ir::DefKind::Struct => false,
                    ir::DefKind::Impl => false,
                    ir::DefKind::TyParam(_) => false,
                    ir::DefKind::Ctor(..) => false,
                },
                ir::Res::Err => false,
                ir::Res::PrimTy(_) => unreachable!(),
            },
            ExprKind::Field(..) | ExprKind::Unary(UnaryOp::Deref, _) => true,
            _ => false,
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
    Box(&'ir ir::Expr<'ir>),
    /// named field access `foo.x` or `tuple.1`
    Field(&'ir ir::Expr<'ir>, Ident),
}

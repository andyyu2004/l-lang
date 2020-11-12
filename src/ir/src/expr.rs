use crate::{self as ir, DefKind, QPath, Res};
use ast::{self, Ident, UnaryOp};
use span::Span;

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

impl<'ir> ir::Expr<'ir> {
    pub fn is_syntactic_lvalue(&self) -> bool {
        match self.kind {
            ir::ExprKind::Path(qpath) => match qpath {
                QPath::Resolved(path) => match path.res {
                    Res::Local(..) => true,
                    Res::Def(..) => false,
                    Res::SelfTy { .. } | Res::SelfVal { .. } => false,
                    Res::Err => false,
                    Res::PrimTy(..) => unreachable!(),
                },
                ir::QPath::TypeRelative(..) => todo!(),
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
    Path(&'ir QPath<'ir>),
    Tuple(&'ir [ir::Expr<'ir>]),
    Closure(&'ir ir::FnSig<'ir>, &'ir ir::Body<'ir>),
    Assign(&'ir ir::Expr<'ir>, &'ir ir::Expr<'ir>),
    Call(&'ir ir::Expr<'ir>, &'ir [ir::Expr<'ir>]),
    Match(&'ir ir::Expr<'ir>, &'ir [ir::Arm<'ir>], ir::MatchSource),
    Struct(&'ir QPath<'ir>, &'ir [ir::Field<'ir>]),
    Box(&'ir ir::Expr<'ir>),
    /// named field access `foo.x` or `tuple.1`
    Field(&'ir ir::Expr<'ir>, Ident),
    Err,
}

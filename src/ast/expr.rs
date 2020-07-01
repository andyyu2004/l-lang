use super::Span;
use crate::{
    error::ParseResult, parser::{Parse, Parser}
};

pub struct Expr {
    span: Span,
    kind: ExprKind,
}

pub struct Lit {}

pub enum ExprKind {
    Lit(Lit),
}

impl Parse for Expr {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        todo!()
    }
}

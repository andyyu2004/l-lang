use super::*;
use crate::{
    ast::*, ctx::Ctx, error::{ParseError, ParseResult}, lexer::{LiteralKind, Span, Tok, TokenKind}
};
use itertools::Itertools;

crate struct Parser<'ctx> {
    tokens: Vec<Tok>,
    idx: usize,
    pub(super) ctx: &'ctx Ctx,
}

impl<'ctx> Parser<'ctx> {
    pub fn new<I>(ctx: &'ctx Ctx, tokens: I) -> Self
    where
        I: IntoIterator<Item = Tok>,
    {
        Self {
            tokens: tokens.into_iter().collect(),
            ctx,
            idx: 0,
        }
    }

    pub fn parse(&mut self) -> ParseResult<Prog> {
        todo!()
    }

    pub fn parse_item(&mut self) -> ParseResult<Item> {
        todo!()
    }

    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        ExprParser.parse(self)
    }

    pub(super) fn accept_one_of<'i, I>(&mut self, kinds: &'i I) -> Option<Tok>
    where
        &'i I: IntoIterator<Item = &'i TokenKind>,
    {
        let res = kinds
            .into_iter()
            .fold(None, |acc, &k| acc.or(self.accept(k)));
        return res;
    }

    pub(super) fn next(&mut self) -> Tok {
        let tok = self.peek();
        self.idx += 1;
        tok
    }

    fn safe_peek(&self) -> Option<Tok> {
        if self.idx < self.tokens.len() {
            Some(self.tokens[self.idx])
        } else {
            None
        }
    }

    pub(super) fn peek(&self) -> Tok {
        self.safe_peek().unwrap()
    }

    pub(super) fn accept(&mut self, kind: TokenKind) -> Option<Tok> {
        self.safe_peek().and_then(|t| {
            if t.kind == kind {
                self.idx += 1;
                Some(t)
            } else {
                None
            }
        })
    }

    pub(super) fn expect(&mut self, kind: TokenKind) -> ParseResult<Tok> {
        let t = self.peek();
        if t.kind == kind {
            self.idx += 1;
            Ok(t)
        } else {
            Err(ParseError::expected(kind, t))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Driver;

    macro_rules! parse_expr {
        ($src:expr) => {{
            let driver = Driver::new($src);
            driver.parse_expr().unwrap()
        }};
    }

    #[test]
    fn parse_int_literal() {
        let expr = parse_expr!("2");
        assert_eq!(expr, Expr::new(Span::new(0, 1), ExprKind::Lit(Lit::Int(2))));
    }

    #[test]
    fn parse_simple_binary_expr() {
        let expr = parse_expr!("2 + 3");
        assert_eq!(
            expr,
            Expr::new(
                Span::new(0, 5),
                ExprKind::Bin(
                    BinOp::Add,
                    box Expr::new(Span::new(0, 1), ExprKind::Lit(Lit::Int(2))),
                    box Expr::new(Span::new(4, 5), ExprKind::Lit(Lit::Int(3))),
                )
            )
        );
    }

    #[test]
    fn parse_precedence_expr() {
        let expr = parse_expr!("2 + 3 * 4");
        assert_eq!(
            expr,
            Expr::new(
                Span::new(0, 9),
                ExprKind::Bin(
                    BinOp::Add,
                    box Expr::new(Span::new(0, 1), ExprKind::Lit(Lit::Int(2))),
                    box Expr::new(
                        Span::new(4, 9),
                        ExprKind::Bin(
                            BinOp::Mul,
                            box Expr::new(Span::new(4, 5), ExprKind::Lit(Lit::Int(3))),
                            box Expr::new(Span::new(8, 9), ExprKind::Lit(Lit::Int(4))),
                        )
                    ),
                )
            )
        );
    }
}

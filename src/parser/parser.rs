use super::Parse;
use crate::{
    ast::*, ctx::Ctx, error::{ParseError, ParseResult}, lexer::{LiteralKind, Span, Tok, TokenKind}
};
use itertools::Itertools;

const UNARY_OPS: [TokenKind; 2] = [TokenKind::Not, TokenKind::Minus];
const TERM_OPS: [TokenKind; 2] = [TokenKind::Plus, TokenKind::Minus];
const FACTOR_OPS: [TokenKind; 2] = [TokenKind::Star, TokenKind::Slash];

crate struct Parser<'ctx> {
    tokens: Vec<Tok>,
    idx: usize,
    ctx: &'ctx Ctx,
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
        Item::parse(self)
    }

    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_term()
    }

    pub fn parse_term(&mut self) -> ParseResult<Expr> {
        self.parse_binary(Self::parse_factor, &TERM_OPS)
    }

    pub fn parse_factor(&mut self) -> ParseResult<Expr> {
        self.parse_binary(Self::parse_unary, &FACTOR_OPS)
    }

    // for<'r, 'b> fn(&'r mut Parser<'b>, Token<'b>) -> Result<(ExprKind, Option<Ty>), Error>;
    //
    pub fn parse_binary<'i, P, I>(&mut self, mut parse: P, ops: &'i I) -> ParseResult<Expr>
    where
        P: FnMut(&mut Parser<'ctx>) -> ParseResult<Expr>,
        &'i I: IntoIterator<Item = &'i TokenKind>,
    {
        let mut expr = parse(self)?;
        while let Some(t) = self.accept_one_of(ops) {
            let binop = BinOp::from(t);
            let right = parse(self)?;
            let span = expr.span.merge(&right.span);
            expr = Expr::new(span, ExprKind::Bin(binop, box expr, box right));
        }
        Ok(expr)
    }

    pub fn parse_unary(&mut self) -> ParseResult<Expr> {
        if let Some(t) = self.accept_one_of(&UNARY_OPS) {
            let unary_op = UnaryOp::from(t);
            let expr = box self.parse_unary()?;
            Ok(Expr::new(
                t.span.merge(&expr.span),
                ExprKind::Unary(unary_op, expr),
            ))
        } else {
            self.parse_primary()
        }
    }

    pub fn parse_literal(&mut self, lit: LiteralKind, span: Span) -> ParseResult<Expr> {
        let literal = match lit {
            LiteralKind::Int { base, .. } => {
                let slice = &self.ctx.main_file()[span];
                println!("slice: {}", slice);
                Lit::Int(i64::from_str_radix(slice, base as u32).unwrap())
            }
            _ => todo!(),
        };
        Ok(Expr::new(span, ExprKind::Lit(literal)))
    }

    pub fn parse_primary(&mut self) -> ParseResult<Expr> {
        if let Some(lparen) = self.accept(TokenKind::OpenParen) {
            let expr = box self.parse_expr()?;
            let rparen = self.expect(TokenKind::CloseParen)?;
            let span = lparen.span.merge(&rparen.span);
            Ok(Expr::new(span, ExprKind::Paren(expr)))
        } else if let TokenKind::Literal { kind, .. } = self.peek().kind {
            let span = self.next().span;
            Ok(self.parse_literal(kind, span)?)
        } else {
            todo!()
        }
    }

    fn accept_one_of<'i, I>(&mut self, kinds: &'i I) -> Option<Tok>
    where
        &'i I: IntoIterator<Item = &'i TokenKind>,
    {
        let res = kinds
            .into_iter()
            .fold(None, |acc, &k| acc.or(self.accept(k)));
        return res;
    }

    fn next(&mut self) -> Tok {
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

    fn peek(&self) -> Tok {
        self.safe_peek().unwrap()
    }

    fn accept(&mut self, kind: TokenKind) -> Option<Tok> {
        self.safe_peek().and_then(|t| {
            if t.kind == kind {
                self.idx += 1;
                Some(t)
            } else {
                None
            }
        })
    }

    pub fn expect(&mut self, kind: TokenKind) -> ParseResult<Tok> {
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

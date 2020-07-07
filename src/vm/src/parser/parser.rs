use super::*;
use crate::ast::*;
use crate::error::*;
use crate::lexer::*;
use crate::span::{self, Span};

crate struct Parser<'ctx> {
    tokens: Vec<Tok>,
    idx: usize,
    pub(super) ctx: &'ctx span::Ctx,
}

impl<'ctx> Parser<'ctx> {
    pub fn new<I>(ctx: &'ctx span::Ctx, tokens: I) -> Self
    where
        I: IntoIterator<Item = Tok>,
    {
        Self { tokens: tokens.into_iter().collect(), ctx, idx: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Prog> {
        todo!()
    }

    pub fn parse_item(&mut self) -> ParseResult<Item> {
        ItemParser.parse(self)
    }

    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        ExprParser.parse(self)
    }

    pub(super) fn accept_one_of<'i, I>(&mut self, ttypes: &'i I) -> Option<Tok>
    where
        &'i I: IntoIterator<Item = &'i TokenType>,
    {
        ttypes.into_iter().fold(None, |acc, &t| acc.or(self.accept(t)))
    }

    pub(super) fn try_parse<R>(&mut self, mut parser: impl Parse<Output = R>) -> Option<R> {
        let backtrack_idx = self.idx;
        parser.parse(self).ok().or_else(|| {
            self.idx = backtrack_idx;
            None
        })
    }

    pub(super) fn next(&mut self) -> Tok {
        let tok = self.peek();
        self.idx += 1;
        tok
    }

    pub(super) fn safe_peek(&self) -> Option<Tok> {
        if self.idx < self.tokens.len() { Some(self.tokens[self.idx]) } else { None }
    }

    pub(super) fn safe_peek_ttype(&self) -> Option<TokenType> {
        self.safe_peek().map(|t| t.ttype)
    }

    pub(super) fn peek(&self) -> Tok {
        self.safe_peek().unwrap()
    }

    pub(super) fn accept_literal(&mut self) -> Option<(LiteralKind, Span)> {
        let Tok { span, ttype } = self.safe_peek()?;
        match ttype {
            TokenType::Literal { kind, .. } => {
                self.idx += 1;
                Some((kind, span))
            }
            _ => None,
        }
    }

    pub(super) fn accept_ident(&mut self) -> Option<(Symbol, Span)> {
        let Tok { span, ttype } = self.safe_peek()?;
        match ttype {
            TokenType::Ident(ident) => {
                self.idx += 1;
                Some((ident, span))
            }
            _ => None,
        }
    }

    pub(super) fn accept(&mut self, ttype: TokenType) -> Option<Tok> {
        self.safe_peek().and_then(|t| {
            if t.ttype == ttype {
                self.idx += 1;
                Some(t)
            } else {
                None
            }
        })
    }

    pub(super) fn expect(&mut self, ttype: TokenType) -> ParseResult<Tok> {
        let t = self.peek();
        if t.ttype == ttype {
            self.idx += 1;
            Ok(t)
        } else {
            Err(ParseError::expected(ttype, t))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{span::Span, Driver};

    macro_rules! parse_expr {
        ($src:expr) => {{
            let driver = Driver::new($src);
            driver.parse_expr().unwrap()
        }};
    }

    #[test]
    fn parse_int_literal() {
        let expr = parse_expr!("2");
        assert_eq!(expr, Expr::new(Span::new(0, 1), ExprKind::Lit(Lit::Num(2.0))));
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
                    box Expr::new(Span::new(0, 1), ExprKind::Lit(Lit::Num(2.0))),
                    box Expr::new(Span::new(4, 5), ExprKind::Lit(Lit::Num(3.0))),
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
                    box Expr::new(Span::new(0, 1), ExprKind::Lit(Lit::Num(2.0))),
                    box Expr::new(
                        Span::new(4, 9),
                        ExprKind::Bin(
                            BinOp::Mul,
                            box Expr::new(Span::new(4, 5), ExprKind::Lit(Lit::Num(3.0))),
                            box Expr::new(Span::new(8, 9), ExprKind::Lit(Lit::Num(4.0))),
                        )
                    ),
                )
            )
        );
    }
}

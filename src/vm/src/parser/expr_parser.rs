use super::*;
use crate::ast::*;
use crate::error::*;
use crate::lexer::*;
use crate::span::Span;

const UNARY_OPS: [TokenType; 2] = [TokenType::Not, TokenType::Minus];
const TERM_OPS: [TokenType; 2] = [TokenType::Plus, TokenType::Minus];
const FACTOR_OPS: [TokenType; 2] = [TokenType::Star, TokenType::Slash];

pub(super) struct ExprParser;

impl Parse for ExprParser {
    type Output = P<Expr>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        TermExprParser.parse(parser)
    }
}

pub(super) struct TermExprParser;

impl Parse for TermExprParser {
    type Output = P<Expr>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        LBinaryExprParser { ops: &TERM_OPS, inner: FactorExprParser }.parse(parser)
    }
}

pub(super) struct FactorExprParser;

impl Parse for FactorExprParser {
    type Output = P<Expr>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        LBinaryExprParser { ops: &FACTOR_OPS, inner: UnaryExprParser }.parse(parser)
    }
}

pub(super) struct UnaryExprParser;

impl Parse for UnaryExprParser {
    type Output = P<Expr>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(t) = parser.accept_one_of(&UNARY_OPS) {
            let unary_op = UnaryOp::from(t);
            let expr = self.parse(parser)?;
            Ok(parser.mk_expr(t.span.merge(&expr.span), ExprKind::Unary(unary_op, expr)))
        } else {
            PrimaryExprParser.parse(parser)
        }
    }
}

pub(super) struct PrimaryExprParser;

impl Parse for PrimaryExprParser {
    type Output = P<Expr>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(open_paren) = parser.accept(TokenType::OpenParen) {
            let (expr, span) = ParenParser { open_paren, inner: ExprParser }.parse(parser)?;
            Ok(parser.mk_expr(span, ExprKind::Paren(expr)))
        } else if let Some((kind, span)) = parser.accept_literal() {
            LiteralExprParser { kind, span }.parse(parser)
        } else if let TokenType::Ident(_) = parser.safe_peek()?.ttype {
            let path = PathParser.parse(parser)?;
            Ok(parser.mk_expr(path.span, ExprKind::Path(path)))
        } else if let Some(tok) = parser.accept(TokenType::False) {
            Ok(parser.mk_expr(tok.span, ExprKind::Lit(Lit::Bool(false))))
        } else if let Some(tok) = parser.accept(TokenType::True) {
            Ok(parser.mk_expr(tok.span, ExprKind::Lit(Lit::Bool(true))))
        } else if let Some(open_brace) = parser.accept(TokenType::OpenBrace) {
            let block = BlockParser { open_brace }.parse(parser)?;
            Ok(parser.mk_expr(block.span, ExprKind::Block(block)))
        } else {
            todo!("found token {:?}", parser.safe_peek());
        }
    }
}

pub(super) struct LiteralExprParser {
    kind: LiteralKind,
    span: Span,
}

impl Parse for LiteralExprParser {
    type Output = P<Expr>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let literal = match self.kind {
            LiteralKind::Int { base, .. } | LiteralKind::Float { base, .. } => {
                let slice = &parser.ctx.main_file()[self.span];
                Lit::Num(i64::from_str_radix(slice, base as u32).unwrap() as f64)
            }
            _ => todo!(),
        };
        Ok(parser.mk_expr(self.span, ExprKind::Lit(literal)))
    }
}

/// left associative binary expr parse
pub(super) struct LBinaryExprParser<'i, Q, I> {
    ops: &'i I,
    inner: Q,
}

impl<'i, Q, I> Parse for LBinaryExprParser<'i, Q, I>
where
    &'i I: IntoIterator<Item = &'i TokenType>,
    Q: Parse<Output = P<Expr>>,
{
    type Output = P<Expr>;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut expr = self.inner.parse(parser)?;
        while let Some(t) = parser.accept_one_of(self.ops) {
            let binop = BinOp::from(t);
            let right = self.inner.parse(parser)?;
            let span = expr.span.merge(&right.span);
            expr = parser.mk_expr(span, ExprKind::Bin(binop, expr, right));
        }
        Ok(expr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{span::Span, Driver};
    use indexed_vec::Idx;

    macro_rules! parse_expr {
        ($src:expr) => {{
            let driver = Driver::new($src);
            driver.parse_expr().unwrap()
        }};
    }

    #[test]
    fn parse_int_literal() {
        let expr = parse_expr!("2");
        assert_eq!(
            expr,
            box Expr::new(Span::new(0, 1), NodeId::new(0), ExprKind::Lit(Lit::Num(2.0)))
        );
    }

    #[test]
    fn parse_simple_binary_expr() {
        let expr = parse_expr!("2 + 3");
        assert_eq!(
            expr,
            box Expr::new(
                Span::new(0, 5),
                NodeId::new(2),
                ExprKind::Bin(
                    BinOp::Add,
                    box Expr::new(Span::new(0, 1), NodeId::new(0), ExprKind::Lit(Lit::Num(2.0))),
                    box Expr::new(Span::new(4, 5), NodeId::new(1), ExprKind::Lit(Lit::Num(3.0))),
                )
            )
        );
    }

    #[test]
    fn parse_precedence_expr() {
        let expr = parse_expr!("2 + 3 * 4");
        assert_eq!(
            expr,
            box Expr::new(
                Span::new(0, 9),
                NodeId::new(4),
                ExprKind::Bin(
                    BinOp::Add,
                    box Expr::new(Span::new(0, 1), NodeId::new(0), ExprKind::Lit(Lit::Num(2.0))),
                    box Expr::new(
                        Span::new(4, 9),
                        NodeId::new(3),
                        ExprKind::Bin(
                            BinOp::Mul,
                            box Expr::new(
                                Span::new(4, 5),
                                NodeId::new(1),
                                ExprKind::Lit(Lit::Num(3.0))
                            ),
                            box Expr::new(
                                Span::new(8, 9),
                                NodeId::new(2),
                                ExprKind::Lit(Lit::Num(4.0))
                            ),
                        )
                    ),
                )
            )
        );
    }
}

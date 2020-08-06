use super::*;
use crate::ast::*;
use crate::error::*;
use crate::lexer::*;
use crate::span::Span;

const UNARY_OPS: [TokenType; 2] = [TokenType::Not, TokenType::Minus];
const POSTFIX_OPS: [TokenType; 3] =
    [TokenType::Dot, TokenType::OpenSqBracket, TokenType::OpenParen];
const CMP_OPS: [TokenType; 2] = [TokenType::Lt, TokenType::Gt];
const TERM_OPS: [TokenType; 2] = [TokenType::Plus, TokenType::Minus];
const FACTOR_OPS: [TokenType; 2] = [TokenType::Star, TokenType::Slash];

// expr parsers are written in increasing order of precedence

pub(super) struct ExprParser;

impl Parse for ExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        CmpExprParser.parse(parser)
    }
}

pub(super) struct CmpExprParser;

impl Parse for CmpExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        LBinaryExprParser { ops: &CMP_OPS, inner: TermExprParser }.parse(parser)
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
            Ok(parser.mk_expr(t.span.merge(expr.span), ExprKind::Unary(unary_op, expr)))
        } else {
            PostfixExprParser.parse(parser)
        }
    }
}

/// parses field accesses, function calls, and index expressions
/// these are all left associative
pub(super) struct PostfixExprParser;

impl Parse for PostfixExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut expr = PrimaryExprParser.parse(parser)?;
        while let Some(t) = parser.accept_one_of(&POSTFIX_OPS) {
            match t.ttype {
                TokenType::OpenParen => {
                    let (arg_span, args) =
                        TupleParser { inner: ExprParser }.spanned(true).parse(parser)?;
                    expr = parser.mk_expr(expr.span.merge(arg_span), ExprKind::Call(expr, args));
                }
                TokenType::Dot => todo!(),
                TokenType::OpenSqBracket => todo!(),
                _ => unreachable!(),
            }
        }
        Ok(expr)
    }
}

pub(super) struct PrimaryExprParser;

impl Parse for PrimaryExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(open_paren) = parser.accept(TokenType::OpenParen) {
            // we first try to parse as a parenthesization, if there is a comma then it will fail
            // and we will backtrack and parse it as a tuple instead
            let mut paren_parser = ParenParser { inner: ExprParser }.spanned(true);
            if let Some((span, expr)) = parser.try_parse(&mut paren_parser) {
                Ok(parser.mk_expr(span, ExprKind::Paren(expr)))
            } else {
                let mut tuple_parser = TupleParser { inner: ExprParser }.spanned(true);
                let (span, elements) = tuple_parser.parse(parser)?;
                Ok(parser.mk_expr(span, ExprKind::Tuple(elements)))
            }
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
        } else if let Some(fn_kw) = parser.accept(TokenType::Fn) {
            LambdaParser { fn_kw }.parse(parser)
        } else if let Some(if_kw) = parser.accept(TokenType::If) {
            IfParser { if_kw }.parse(parser)
        } else {
            Err(ParseError::unimpl())
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
        let slice = &parser.ctx.main_file()[self.span];
        let literal = match self.kind {
            LiteralKind::Float { base, .. } => {
                if base != Base::Decimal {
                    panic!("only decimal float literals are supported")
                }
                Lit::Num(slice.parse().unwrap())
            }
            LiteralKind::Int { base, .. } =>
                Lit::Num(i64::from_str_radix(slice, base as u32).unwrap() as f64),
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
            let span = expr.span.merge(right.span);
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

    macro parse_expr($src:expr) {{
        let driver = Driver::new($src);
        driver.parse_expr().unwrap()
    }}

    macro fmt_expr($src:expr) {{
        let expr = parse_expr!($src);
        format!("{}", expr)
    }}

    #[test]
    fn parse_nested_if() {
        let expr = parse_expr!("if false { 5 } else if true { 6 } else { 7 }");
        dbg!(expr);
    }

    #[test]
    fn parse_call_expr() {
        let expr = parse_expr!("f(2,3,x)");
    }

    #[test]
    fn parse_left_assoc_call_expr() {
        let expr = fmt_expr!("1(2)(3)(4)");
        assert_eq!(expr, "(((1 2) 3) 4)")
    }

    #[test]
    fn test_parser_span() {
        let expr = parse_expr!("    3");
        dbg!(&expr);
        assert_eq!(
            expr,
            box Expr::new(Span::new(4, 5), NodeId::new(0), ExprKind::Lit(Lit::Num(3.0)))
        );
    }

    #[test]
    fn parse_empty_tuple() {
        let expr = parse_expr!("()");
        assert_eq!(expr, box Expr::new(Span::new(0, 2), NodeId::new(0), ExprKind::Tuple(vec![])));
    }

    #[test]
    fn parse_tuple() {
        let expr = parse_expr!("(2, 3)");
        assert_eq!(
            expr,
            box Expr::new(
                Span::new(0, 6),
                NodeId::new(2),
                ExprKind::Tuple(vec![
                    box Expr::new(Span::new(1, 2), NodeId::new(0), ExprKind::Lit(Lit::Num(2.0))),
                    box Expr::new(Span::new(4, 5), NodeId::new(1), ExprKind::Lit(Lit::Num(3.0)))
                ])
            )
        );
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
    fn parse_parameterless_lambda() {
        parse_expr!("fn () => 5");
    }

    #[test]
    fn parse_lambda() {
        let _expr = parse_expr!("fn (x, y) => (2,3,4)");
    }

    #[test]
    fn parse_typed_lambda() {
        let _expr = parse_expr!("fn (x: i64, y: f64) => (2,3,4)");
    }

    #[test]
    fn parse_typed_lambda_with_ret_ty() {
        let _expr = parse_expr!("fn (x: i64, y: f64) -> (u64, u64, u64) => (2,3,4)");
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

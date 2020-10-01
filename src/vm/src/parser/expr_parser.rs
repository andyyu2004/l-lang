use super::*;
use crate::ast::*;
use crate::error::*;
use crate::lexer::*;
use crate::span::{with_source_map, Span};

const UNARY_OPS: [TokenType; 4] =
    [TokenType::Not, TokenType::Minus, TokenType::Star, TokenType::And];
const POSTFIX_OPS: [TokenType; 3] =
    [TokenType::Dot, TokenType::OpenSqBracket, TokenType::OpenParen];
const CMP_OPS: [TokenType; 2] = [TokenType::Lt, TokenType::Gt];
const TERM_OPS: [TokenType; 2] = [TokenType::Plus, TokenType::Minus];
const FACTOR_OPS: [TokenType; 2] = [TokenType::Star, TokenType::Slash];

// expr parsers are written in increasing order of precedence

pub(super) struct ExprParser;

impl<'a> Parse<'a> for ExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        BoxExprParser.parse(parser)
    }
}

// TODO maybe this low precedence for box is dumb?
struct BoxExprParser;

impl<'a> Parse<'a> for BoxExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        if let Some(kw) = parser.accept(TokenType::Box) {
            let expr = parser.parse_expr()?;
            Ok(parser.mk_expr(kw.span.merge(expr.span), ExprKind::Box(expr)))
        } else {
            AssnExprParser.parse(parser)
        }
    }
}

struct AssnExprParser;

impl<'a> Parse<'a> for AssnExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let mut expr = CmpExprParser.parse(parser)?;
        while let Some(eq) = parser.accept(TokenType::Eq) {
            let right = self.parse(parser)?;
            expr = parser.mk_expr(expr.span.merge(right.span), ExprKind::Assign(expr, right));
        }
        Ok(expr)
    }
}

struct CmpExprParser;

impl<'a> Parse<'a> for CmpExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        LBinaryExprParser { ops: &CMP_OPS, inner: TermExprParser }.parse(parser)
    }
}

struct TermExprParser;

impl<'a> Parse<'a> for TermExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        LBinaryExprParser { ops: &TERM_OPS, inner: FactorExprParser }.parse(parser)
    }
}

struct FactorExprParser;

impl<'a> Parse<'a> for FactorExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        LBinaryExprParser { ops: &FACTOR_OPS, inner: UnaryExprParser }.parse(parser)
    }
}

pub(super) struct UnaryExprParser;

impl<'a> Parse<'a> for UnaryExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        if let Some(t) = parser.accept_one_of(&UNARY_OPS) {
            let unary_op = UnaryOp::from(t);
            let expr = self.parse(parser)?;
            let span = t.span.merge(expr.span);
            if unary_op == UnaryOp::Ref && !parser.in_unsafe_ctx() {
                parser.sess.emit_error(span, ParseError::RequireUnsafeCtx);
            }
            Ok(parser.mk_expr(span, ExprKind::Unary(unary_op, expr)))
        } else {
            PostfixExprParser.parse(parser)
        }
    }
}

/// parses field accesses, function calls, and index expressions
/// these are all left associative
struct PostfixExprParser;

impl<'a> Parse<'a> for PostfixExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let mut expr = PrimaryExprParser.parse(parser)?;
        while let Some(t) = parser.accept_one_of(&POSTFIX_OPS) {
            match t.ttype {
                TokenType::OpenParen => {
                    let (arg_span, args) =
                        TupleParser { inner: ExprParser }.spanned(true).parse(parser)?;
                    expr = parser.mk_expr(expr.span.merge(arg_span), ExprKind::Call(expr, args));
                }
                TokenType::Dot => expr = FieldAccessParser { expr }.parse(parser)?,
                TokenType::OpenSqBracket => todo!(),
                _ => unreachable!(),
            }
        }
        Ok(expr)
    }
}

struct PrimaryExprParser;

impl<'a> Parse<'a> for PrimaryExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
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
            LiteralParser { kind, span }.parse(parser)
        } else if let Some(ret_kw) = parser.accept(TokenType::Return) {
            RetParser { ret_kw }.parse(parser)
        } else if let TokenType::Ident(_) = parser.safe_peek()?.ttype {
            PathExprParser.parse(parser)
        } else if let Some(tok) = parser.accept(TokenType::False) {
            Ok(parser.mk_expr(tok.span, ExprKind::Lit(Lit::Bool(false))))
        } else if let Some(tok) = parser.accept(TokenType::True) {
            Ok(parser.mk_expr(tok.span, ExprKind::Lit(Lit::Bool(true))))
        } else if let Some(open_brace) = parser.accept(TokenType::OpenBrace) {
            let block = BlockParser { open_brace }.parse(parser)?;
            Ok(parser.mk_expr(block.span, ExprKind::Block(block)))
        } else if let Some(fn_kw) = parser.accept(TokenType::Fn) {
            ClosureParser { fn_kw }.parse(parser)
        } else if let Some(if_kw) = parser.accept(TokenType::If) {
            IfParser { if_kw }.parse(parser)
        } else if let Some(unsafe_kw) = parser.accept(TokenType::Unsafe) {
            let open_brace = parser.expect(TokenType::OpenBrace)?;
            let blk = parser.enter_unsafe_ctx(|parser| BlockParser { open_brace }.parse(parser))?;
            Ok(parser.mk_expr(unsafe_kw.span.merge(blk.span), ExprKind::Block(blk)))
        } else if let Some(match_kw) = parser.accept(TokenType::Match) {
            MatchParser { match_kw }.parse(parser)
        } else {
            Err(parser.err(parser.empty_span(), ParseError::Unimpl))
        }
    }
}

/// left associative binary expr parse
pub(super) struct LBinaryExprParser<'i, Q, I> {
    ops: &'i I,
    inner: Q,
}

impl<'a, 'i, Q, I> Parse<'a> for LBinaryExprParser<'i, Q, I>
where
    &'i I: IntoIterator<Item = &'i TokenType>,
    Q: Parse<'a, Output = P<Expr>>,
{
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
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
    fn parse_deref() {
        parse_expr!("*x");
    }

    #[test]
    fn parse_ref() {
        parse_expr!("&x");
    }

    #[test]
    fn parse_chained_tuple_accesses() {
        // parse_expr!("x.1.1");
        // parse_expr!("x.1.1.1");
        parse_expr!("x.0.1.2.3.4.5.6");
    }

    #[test]
    fn parse_assign() {
        let _expr = parse_expr!("x = y");
        let _expr = parse_expr!("x = y = 2");
        dbg!(_expr);
    }

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
            box Expr::new(Span::new(4, 5), NodeId::new(0), ExprKind::Lit(Lit::Int(3)))
        );
    }

    #[test]
    fn parse_empty_tuple() {
        let expr = parse_expr!("()");
        assert_eq!(expr, box Expr::new(Span::new(0, 2), NodeId::new(0), ExprKind::Tuple(vec![])));
    }

    #[test]
    fn parse_struct_expr() {
        let _expr = parse_expr!("SomeStruct { x: int, y: bool }");
        dbg!(_expr);
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
                    box Expr::new(Span::new(1, 2), NodeId::new(0), ExprKind::Lit(Lit::Int(2))),
                    box Expr::new(Span::new(4, 5), NodeId::new(1), ExprKind::Lit(Lit::Int(3)))
                ])
            )
        );
    }

    #[test]
    fn parse_int_literal() {
        let expr = parse_expr!("2");
        assert_eq!(
            expr,
            box Expr::new(Span::new(0, 1), NodeId::new(0), ExprKind::Lit(Lit::Int(2)))
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
                    box Expr::new(Span::new(0, 1), NodeId::new(0), ExprKind::Lit(Lit::Int(2))),
                    box Expr::new(Span::new(4, 5), NodeId::new(1), ExprKind::Lit(Lit::Int(3))),
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
                    box Expr::new(Span::new(0, 1), NodeId::new(0), ExprKind::Lit(Lit::Int(2))),
                    box Expr::new(
                        Span::new(4, 9),
                        NodeId::new(3),
                        ExprKind::Bin(
                            BinOp::Mul,
                            box Expr::new(
                                Span::new(4, 5),
                                NodeId::new(1),
                                ExprKind::Lit(Lit::Int(3))
                            ),
                            box Expr::new(
                                Span::new(8, 9),
                                NodeId::new(2),
                                ExprKind::Lit(Lit::Int(4))
                            ),
                        )
                    ),
                )
            )
        );
    }
}

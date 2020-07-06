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
    type Output = Expr;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        TermExprParser.parse(parser)
    }
}

pub(super) struct TermExprParser;

impl Parse for TermExprParser {
    type Output = Expr;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        LBinaryExprParser {
            ops: &TERM_OPS,
            inner: FactorExprParser,
        }
        .parse(parser)
    }
}

pub(super) struct FactorExprParser;

impl Parse for FactorExprParser {
    type Output = Expr;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        LBinaryExprParser {
            ops: &FACTOR_OPS,
            inner: UnaryExprParser,
        }
        .parse(parser)
    }
}

pub(super) struct UnaryExprParser;

impl Parse for UnaryExprParser {
    type Output = Expr;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(t) = parser.accept_one_of(&UNARY_OPS) {
            let unary_op = UnaryOp::from(t);
            let expr = box self.parse(parser)?;
            Ok(Expr::new(
                t.span.merge(&expr.span),
                ExprKind::Unary(unary_op, expr),
            ))
        } else {
            PrimaryExprParser.parse(parser)
        }
    }
}

pub(super) struct PrimaryExprParser;

impl Parse for PrimaryExprParser {
    type Output = Expr;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(lparen) = parser.accept(TokenType::OpenParen) {
            let expr = box ExprParser.parse(parser)?;
            let rparen = parser.expect(TokenType::CloseParen)?;
            let span = lparen.span.merge(&rparen.span);
            Ok(Expr::new(span, ExprKind::Paren(expr)))
        } else if let Some((kind, span)) = parser.accept_literal() {
            LiteralExprParser { kind, span }.parse(parser)
        } else if let Some((ident, span)) = parser.accept_ident() {
            IdentParser { ident, span }.parse(parser)
        } else if let Some(tok) = parser.accept(TokenType::False) {
            Ok(Expr::new(tok.span, ExprKind::Lit(Lit::Bool(false))))
        } else if let Some(tok) = parser.accept(TokenType::True) {
            Ok(Expr::new(tok.span, ExprKind::Lit(Lit::Bool(true))))
        } else {
            todo!()
        }
    }
}

pub(super) struct IdentParser {
    span: Span,
    ident: Symbol,
}

impl Parse for IdentParser {
    type Output = Expr;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        // Ok(Expr::new(self.span, kind));
        todo!()
    }
}

pub(super) struct LiteralExprParser {
    kind: LiteralKind,
    span: Span,
}

impl Parse for LiteralExprParser {
    type Output = Expr;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let literal = match self.kind {
            LiteralKind::Int { base, .. } | LiteralKind::Float { base, .. } => {
                let slice = &parser.ctx.main_file()[self.span];
                Lit::Num(i64::from_str_radix(slice, base as u32).unwrap() as f64)
            }
            _ => todo!(),
        };
        Ok(Expr::new(self.span, ExprKind::Lit(literal)))
    }
}

/// left associative binary expr parse
pub(super) struct LBinaryExprParser<'i, P, I> {
    ops: &'i I,
    inner: P,
}

impl<'i, P, I> Parse for LBinaryExprParser<'i, P, I>
where
    &'i I: IntoIterator<Item = &'i TokenType>,
    P: Parse<Output = Expr>,
{
    type Output = Expr;
    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut expr = self.inner.parse(parser)?;
        while let Some(t) = parser.accept_one_of(self.ops) {
            let binop = BinOp::from(t);
            let right = self.inner.parse(parser)?;
            let span = expr.span.merge(&right.span);
            expr = Expr::new(span, ExprKind::Bin(binop, box expr, box right));
        }
        Ok(expr)
    }
}

use super::*;
use ast::{Let, Stmt, StmtKind, P};
use lex::{Token, TokenType};

pub struct StmtParser;

impl<'a> Parse<'a> for StmtParser {
    type Output = P<Stmt>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        if let Some(let_kw) = parser.accept(TokenType::Let) {
            LetParser { let_kw }.parse(parser)
        } else {
            let expr = parser.parse_expr();
            if let Some(semi) = parser.accept(TokenType::Semi) {
                Ok(parser.mk_stmt(expr.span.merge(semi.span), StmtKind::Semi(expr)))
            } else {
                Ok(parser.mk_stmt(expr.span, StmtKind::Expr(expr)))
            }
        }
    }
}

/// let <pat>:<ty> = ( <expr> )?;
pub struct LetParser {
    let_kw: Token,
}

impl<'a> Parse<'a> for LetParser {
    type Output = P<Stmt>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let pat = parser.parse_pattern()?;
        let ty = parser.accept(TokenType::Colon).map(|_| parser.parse_ty(true));
        let init = parser.accept(TokenType::Eq).map(|_| parser.parse_expr());
        let semi = parser.expect(TokenType::Semi)?;
        let span = self.let_kw.span.merge(semi.span);
        Ok(parser.mk_stmt(span, StmtKind::Let(box Let { id: parser.mk_id(), span, pat, ty, init })))
    }
}

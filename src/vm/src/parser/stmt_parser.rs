use super::*;
use crate::ast::{Let, Stmt, StmtKind, P};
use crate::{
    error::ParseResult, lexer::{Tok, TokenType}
};

crate struct StmtParser;

impl Parse for StmtParser {
    type Output = P<Stmt>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(let_kw) = parser.accept(TokenType::Let) {
            LetParser { let_kw }.parse(parser)
        } else {
            let expr = ExprParser.parse(parser)?;
            if let Some(semi) = parser.accept(TokenType::Semi) {
                Ok(parser.mk_stmt(expr.span.merge(&semi.span), StmtKind::Semi(expr)))
            } else {
                Ok(parser.mk_stmt(expr.span, StmtKind::Expr(expr)))
            }
        }
    }
}

/// let <pat>:<ty> = ( <expr> )?;
crate struct LetParser {
    let_kw: Tok,
}

impl Parse for LetParser {
    type Output = P<Stmt>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let pat = PatParser.parse(parser)?;
        let ty = parser.accept(TokenType::Colon).map(|_| TyParser.parse(parser)).transpose()?;
        let init = parser.accept(TokenType::Eq).map(|_| ExprParser.parse(parser)).transpose()?;
        let semi = parser.expect(TokenType::Semi)?;
        let span = self.let_kw.span.merge(&semi.span);
        Ok(parser.mk_stmt(span, StmtKind::Let(box Let { id: parser.mk_id(), span, pat, ty, init })))
    }
}

use super::*;
use crate::ast::*;
use crate::error::*;
use crate::lexer::*;
use crate::span::{self, Span};
use indexed_vec::Idx;
use std::cell::Cell;

crate struct Parser<'ctx> {
    tokens: Vec<Tok>,
    idx: usize,
    pub(super) ctx: &'ctx span::Ctx,
    id_counter: Cell<usize>,
}

impl<'ctx> Parser<'ctx> {
    pub fn new<I>(ctx: &'ctx span::Ctx, tokens: I) -> Self
    where
        I: IntoIterator<Item = Tok>,
    {
        Self { tokens: tokens.into_iter().collect(), ctx, idx: 0, id_counter: Cell::new(0) }
    }

    /// runs some parser and returns the result and the span that it consumed
    pub(super) fn with_span<R>(
        &mut self,
        parser: &mut impl Parse<Output = R>,
    ) -> ParseResult<(R, Span)> {
        let lo = self.idx;
        Ok((parser.parse(self)?, Span::new(lo, self.idx)))
    }

    pub fn empty_span(&self) -> Span {
        Span { lo: self.idx, hi: self.idx }
    }

    pub fn parse(&mut self) -> ParseResult<P<Prog>> {
        ProgParser.parse(self)
    }

    pub fn parse_item(&mut self) -> ParseResult<P<Item>> {
        ItemParser.parse(self)
    }

    pub fn parse_expr(&mut self) -> ParseResult<P<Expr>> {
        ExprParser.parse(self)
    }

    pub(super) fn mk_id(&self) -> NodeId {
        let id = self.id_counter.get();
        self.id_counter.set(id + 1);
        NodeId::new(id)
    }

    pub(super) fn try_parse<R, P>(&mut self, parser: &mut P) -> Option<R>
    where
        P: Parse<Output = R>,
    {
        let backtrack_idx = self.idx;
        parser.parse(self).ok().or_else(|| {
            self.idx = backtrack_idx;
            None
        })
    }

    pub(super) fn mk_expr(&self, span: Span, kind: ExprKind) -> P<Expr> {
        box Expr { span, id: self.mk_id(), kind }
    }

    pub(super) fn mk_pat(&self, span: Span, kind: PatternKind) -> P<Pattern> {
        box Pattern { span, id: self.mk_id(), kind }
    }

    pub(super) fn mk_stmt(&self, span: Span, kind: StmtKind) -> P<Stmt> {
        box Stmt { span, id: self.mk_id(), kind }
    }

    pub(super) fn mk_item(
        &self,
        span: Span,
        vis: Visibility,
        ident: Ident,
        kind: ItemKind,
    ) -> P<Item> {
        box Item { span, id: self.mk_id(), ident, vis, kind }
    }

    pub(super) fn next(&mut self) -> Tok {
        let tok = self.peek();
        self.idx += 1;
        tok
    }

    pub(super) fn reached_eof(&self) -> bool {
        self.idx >= self.tokens.len()
    }

    pub(super) fn safe_peek(&self) -> ParseResult<Tok> {
        if !self.reached_eof() {
            Ok(self.tokens[self.idx])
        } else {
            Err(ParseError::unexpected_eof(self.empty_span()))
        }
    }

    pub(super) fn safe_peek_ttype(&self) -> ParseResult<TokenType> {
        self.safe_peek().map(|t| t.ttype)
    }

    pub(super) fn peek(&self) -> Tok {
        self.safe_peek().unwrap()
    }

    pub(super) fn accept_literal(&mut self) -> Option<(LiteralKind, Span)> {
        let Tok { span, ttype } = self.safe_peek().ok()?;
        match ttype {
            TokenType::Literal { kind, .. } => {
                self.idx += 1;
                Some((kind, span))
            }
            _ => None,
        }
    }

    pub(super) fn expect_ident(&mut self) -> ParseResult<Ident> {
        let err_ident = TokenType::Ident(Symbol(0));
        let tok = self.safe_peek()?;
        let Tok { span, ttype } = tok;
        match ttype {
            TokenType::Ident(symbol) => {
                self.idx += 1;
                Ok(Ident { span, id: self.mk_id(), symbol })
            }
            _ => Err(ParseError::expected(err_ident, tok)),
        }
    }

    pub(super) fn accept_ident(&mut self) -> Option<Ident> {
        self.expect_ident().ok()
    }

    pub(super) fn accept(&mut self, ttype: TokenType) -> Option<Tok> {
        self.safe_peek().ok().and_then(|t| {
            if t.ttype == ttype {
                self.idx += 1;
                Some(t)
            } else {
                None
            }
        })
    }

    pub(super) fn accept_one_of<'i, I>(&mut self, ttypes: &'i I) -> Option<Tok>
    where
        &'i I: IntoIterator<Item = &'i TokenType>,
    {
        ttypes.into_iter().fold(None, |acc, &t| acc.or(self.accept(t)))
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

    pub(super) fn expect_one_of<'i, I>(&mut self, ttypes: &'i I) -> ParseResult<Tok>
    where
        &'i I: IntoIterator<Item = &'i TokenType>,
    {
        self.accept_one_of(ttypes).ok_or_else(|| {
            ParseError::expected_one_of(ttypes.into_iter().cloned().collect(), self.peek())
        })
    }
}

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
    /// `include_prev` indicates whether the previous token is to be included in the span or not
    pub(super) fn with_span<R>(
        &mut self,
        parser: &mut impl Parse<Output = R>,
        include_prev: bool,
    ) -> ParseResult<(Span, R)> {
        let lo =
            if include_prev { self.tokens[self.idx - 1].span.lo } else { self.curr_span_start() };
        let p = parser.parse(self)?;
        Ok((Span::new(lo, self.curr_span_start()), p))
    }

    pub fn curr_span_start(&self) -> usize {
        self.tokens[self.idx].span.lo
    }

    pub fn empty_span(&self) -> Span {
        let idx = self.curr_span_start();
        Span { lo: idx, hi: idx }
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
        let backtrack_id = self.id_counter.get();
        parser.parse(self).ok().or_else(|| {
            self.idx = backtrack_idx;
            self.id_counter.set(backtrack_id);
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
        self.tokens[self.idx].ttype == TokenType::Eof
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
                Ok(Ident { span, symbol })
            }
            _ => Err(ParseError::expected(err_ident, tok)),
        }
    }

    pub(super) fn accept_ident(&mut self) -> Option<Ident> {
        self.expect_ident().ok()
    }

    pub(super) fn accept(&mut self, ttype: TokenType) -> Option<Tok> {
        self.expect(ttype).ok()
    }

    pub(super) fn accept_one_of<'i, I>(&mut self, ttypes: &'i I) -> Option<Tok>
    where
        &'i I: IntoIterator<Item = &'i TokenType>,
    {
        ttypes.into_iter().fold(None, |acc, &t| acc.or(self.accept(t)))
    }

    pub(super) fn expect(&mut self, ttype: TokenType) -> ParseResult<Tok> {
        let t = self.safe_peek()?;
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

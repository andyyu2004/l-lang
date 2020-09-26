use super::*;
use crate::ast::*;
use crate::driver::Session;
use crate::error::*;
use crate::lexer::*;
use crate::span::{self, Span};
use indexed_vec::Idx;
use std::cell::Cell;
use std::error::Error;

pub struct Parser<'a> {
    pub sess: &'a Session,
    tokens: Vec<Tok>,
    idx: usize,
    id_counter: Cell<usize>,
    allow_unsafe: bool,
}

impl<'a> Parser<'a> {
    pub fn new<I>(sess: &'a Session, tokens: I) -> Self
    where
        I: IntoIterator<Item = Tok>,
    {
        Self {
            tokens: tokens.into_iter().collect(),
            idx: 0,
            id_counter: Cell::new(0),
            allow_unsafe: false,
            sess,
        }
    }

    pub fn dump_token_stream(&self) {
        for token in &self.tokens[self.idx..] {
            eprintln!("{:?}", token.ttype);
        }
    }

    /// directly moves the parser's cursor backwards by `u` steps
    /// avoid using this unless necessary
    pub fn backtrack(&mut self, u: usize) {
        self.idx -= u;
    }

    pub fn err(&self, span: Span, err: impl Error) -> DiagnosticBuilder<'a> {
        self.sess.build_error(span, err)
    }

    pub fn parse_mutability(&mut self) -> Mutability {
        match self.accept(TokenType::Mut) {
            Some(_) => Mutability::Mut,
            None => Mutability::Imm,
        }
    }

    pub fn in_unsafe_ctx(&self) -> bool {
        self.allow_unsafe
    }

    pub fn enter_unsafe_ctx<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.allow_unsafe = true;
        let ret = f(self);
        self.allow_unsafe = false;
        ret
    }

    /// runs some parser and returns the result and the span that it consumed
    /// `include_prev` indicates whether the previous token is to be included in the span or not
    pub(super) fn with_span<R>(
        &mut self,
        parser: &mut impl Parse<'a, Output = R>,
        include_prev: bool,
    ) -> ParseResult<'a, (Span, R)> {
        let lo =
            if include_prev { self.tokens[self.idx - 1].span.lo } else { self.curr_span_start() };
        let p = parser.parse(self)?;
        Ok((Span::new(lo, self.prev_span_end()), p))
    }

    /// returns true if the current token is an ident
    /// similar to `accept_ident` except the token stream is not advanced
    pub fn ident(&self) -> ParseResult<'a, Option<Ident>> {
        let tok = self.safe_peek()?;
        Ok(if let TokenType::Ident(symbol) = tok.ttype {
            Some(Ident::new(tok.span, symbol))
        } else {
            None
        })
    }

    /// separates float x.y into x . y
    /// assumes the float has been accepted already
    /// returns a pair of the components (x, y) to avoid modifying the token stream
    pub fn split_float(&mut self) -> (Ident, Ident) {
        let token = self.prev();
        let span = token.span;

        let s = match token.ttype {
            TokenType::Literal { kind: LiteralKind::Float { .. }, suffix_start } =>
                span.to_string(),
            _ => unreachable!(),
        };
        let idx = s.find('.').unwrap();
        let x = Symbol::intern(&s[..idx]);
        let xspan = Span::new(span.lo, span.lo + idx);
        let y = Symbol::intern(&s[idx + 1..]);
        let yspan = Span::new(span.lo + idx + 1, span.hi);
        (Ident::new(xspan, x), Ident::new(yspan, y))
    }

    pub fn prev_span_end(&self) -> usize {
        self.tokens[self.idx - 1].span.hi
    }

    pub fn curr_span_start(&self) -> usize {
        self.tokens[self.idx].span.lo
    }

    pub fn empty_span(&self) -> Span {
        let idx = self.curr_span_start();
        Span { lo: idx, hi: idx }
    }

    /// entry point to parsing
    pub fn parse(&mut self) -> Option<P<Prog>> {
        ProgParser.parse(self).map_err(|err| err.emit()).ok()
    }

    pub fn parse_stmt(&mut self) -> ParseResult<'a, P<Stmt>> {
        StmtParser.parse(self)
    }

    pub fn parse_item(&mut self) -> ParseResult<'a, P<Item>> {
        ItemParser.parse(self)
    }

    pub fn parse_expr(&mut self) -> ParseResult<'a, P<Expr>> {
        ExprParser.parse(self)
    }

    pub fn parse_generics(&mut self) -> ParseResult<'a, Generics> {
        GenericsParser.parse(self)
    }

    pub fn parse_ty(&mut self, allow_infer: bool) -> ParseResult<'a, P<Ty>> {
        TyParser { allow_infer }.parse(self)
    }

    pub fn parse_pattern(&mut self) -> ParseResult<'a, P<Pattern>> {
        PatParser.parse(self)
    }

    pub fn parse_type_path(&mut self) -> ParseResult<'a, Path> {
        PathParser { kind: PathKind::Type }.parse(self)
    }

    pub fn parse_path(&mut self) -> ParseResult<'a, Path> {
        PathParser { kind: PathKind::Expr }.parse(self)
    }

    pub(super) fn mk_id(&self) -> NodeId {
        let id = self.id_counter.get();
        self.id_counter.set(id + 1);
        NodeId::new(id)
    }

    pub(super) fn try_parse<R, P>(&mut self, parser: &mut P) -> Option<R>
    where
        P: Parse<'a, Output = R>,
    {
        let backtrack_idx = self.idx;
        let backtrack_id = self.id_counter.get();
        parser.parse(self).ok().or_else(|| {
            self.idx = backtrack_idx;
            self.id_counter.set(backtrack_id);
            None
        })
    }

    pub(super) fn mk_path(&self, span: Span, segments: Vec<PathSegment>) -> Path {
        Path { id: self.mk_id(), span, segments }
    }

    pub(super) fn mk_expr(&self, span: Span, kind: ExprKind) -> P<Expr> {
        box Expr { span, id: self.mk_id(), kind }
    }

    pub(super) fn mk_infer_ty(&self) -> P<Ty> {
        self.mk_ty(self.empty_span(), TyKind::Infer)
    }

    pub(super) fn mk_ty(&self, span: Span, kind: TyKind) -> P<Ty> {
        box Ty { span, id: self.mk_id(), kind }
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

    pub(super) fn bump(&mut self) {
        self.next();
    }

    pub(super) fn next(&mut self) -> Tok {
        let tok = self.peek();
        self.idx += 1;
        tok
    }

    pub(super) fn reached_eof(&self) -> bool {
        self.tokens[self.idx].ttype == TokenType::Eof
    }

    pub(super) fn safe_peek(&self) -> ParseResult<'a, Tok> {
        if !self.reached_eof() {
            Ok(self.tokens[self.idx])
        } else {
            Err(self.err(self.empty_span(), ParseError::Eof))
        }
    }

    pub(super) fn safe_peek_ttype(&self) -> ParseResult<'a, TokenType> {
        self.safe_peek().map(|t| t.ttype)
    }

    pub(super) fn peek(&self) -> Tok {
        self.safe_peek().ok().unwrap()
    }

    pub(super) fn prev(&self) -> Tok {
        self.tokens[self.idx - 1]
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

    pub(super) fn expect_ident(&mut self) -> ParseResult<'a, Ident> {
        let tok = self.safe_peek()?;
        let Tok { span, ttype } = tok;
        match ttype {
            TokenType::Ident(symbol) => {
                self.idx += 1;
                Ok(Ident { span, symbol })
            }
            _ => Err(self.err(tok.span, ParseError::Expected(TokenType::Ident(Symbol(0)), tok))),
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
        ttypes.into_iter().fold(None, |acc, &t| acc.or_else(|| self.accept(t)))
    }

    pub(super) fn expect(&mut self, ttype: TokenType) -> ParseResult<'a, Tok> {
        let t = self.safe_peek()?;
        if t.ttype == ttype {
            self.idx += 1;
            Ok(t)
        } else {
            Err(self.err(t.span, ParseError::Expected(ttype, t)))
        }
    }

    pub(super) fn expect_one_of<'i, I>(&mut self, ttypes: &'i I) -> ParseResult<'a, Tok>
    where
        &'i I: IntoIterator<Item = &'i TokenType>,
    {
        self.accept_one_of(ttypes).ok_or_else(|| {
            let tok = self.peek();
            let err = ParseError::ExpectedOneOf(ttypes.into_iter().cloned().collect(), tok);
            self.err(tok.span, err)
        })
    }
}

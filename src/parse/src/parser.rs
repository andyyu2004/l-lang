use super::*;
use ast::*;
use codespan::{ByteIndex, ByteOffset};
use error::*;
use index::Idx;
use lex::*;
use session::Session;
use span::{self, kw, Span, Symbol};
use std::cell::Cell;
use std::error::Error;

pub struct Parser<'a> {
    pub sess: &'a Session,
    tokens: Vec<Tok>,
    idx: usize,
    id_counter: Cell<usize>,
}

impl<'a> Parser<'a> {
    pub fn new<I>(sess: &'a Session, tokens: I) -> Self
    where
        I: IntoIterator<Item = Tok>,
    {
        Self { tokens: tokens.into_iter().collect(), idx: 0, id_counter: Cell::new(0), sess }
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

    pub fn build_err(&self, span: impl Into<MultiSpan>, err: impl Error) -> DiagnosticBuilder<'a> {
        self.sess.build_error(span, err)
    }

    pub fn parse_mutability(&mut self) -> Mutability {
        match self.accept(TokenType::Mut) {
            Some(_) => Mutability::Mut,
            None => Mutability::Imm,
        }
    }

    /// runs some parser and returns the result and the span that it consumed
    /// `include_prev` indicates whether the previous token is to be included in the span or not
    pub(super) fn with_span<R>(
        &mut self,
        parser: &mut impl Parse<'a, Output = R>,
        include_prev: bool,
    ) -> ParseResult<'a, (Span, R)> {
        let lo = if include_prev {
            self.tokens[self.idx - 1].span.start()
        } else {
            self.curr_span_start()
        };
        let p = parser.parse(self)?;
        Ok((Span::new(lo, self.prev_span_end()), p))
    }

    /// returns true if the current token is an ident
    /// similar to `accept_ident` except the token stream is not advanced
    pub fn is_ident(&self) -> ParseResult<'a, Option<Ident>> {
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
            TokenType::Literal { kind: LiteralKind::Float { .. }, suffix_start: _ } =>
                span.to_string(),
            _ => unreachable!(),
        };
        let idx = s.find('.').unwrap();
        let byte_offset = ByteOffset(idx as i64);
        let x = Symbol::intern(&s[..idx]);
        let xspan = Span::new(span.start(), span.start() + byte_offset);
        let y = Symbol::intern(&s[idx + 1..]);
        let yspan = Span::new(span.start() + byte_offset + 1.into(), span.end());
        (Ident::new(xspan, x), Ident::new(yspan, y))
    }

    pub fn prev_span_end(&self) -> ByteIndex {
        self.tokens[self.idx - 1].span.end()
    }

    pub fn curr_span_start(&self) -> ByteIndex {
        self.tokens[self.idx].span.start()
    }

    pub fn empty_span(&self) -> Span {
        let idx = self.curr_span_start();
        Span::new(idx, idx)
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

    pub fn parse_expr(&mut self) -> P<Expr> {
        ExprParser.parse(self).unwrap_or_else(|err| {
            err.emit();
            self.mk_expr(err.get_span(), ExprKind::Err)
        })
    }

    pub fn parse_generics(&mut self) -> ParseResult<'a, Generics> {
        GenericsParser.parse(self)
    }

    pub fn parse_block(&mut self, open_brace: Tok) -> ParseResult<'a, P<Block>> {
        BlockParser { open_brace, is_unsafe: false }.parse(self)
    }

    pub fn parse_ty(&mut self, allow_infer: bool) -> P<Ty> {
        TyParser { allow_infer }.parse(self).unwrap_or_else(|err| {
            err.emit();
            self.mk_ty(err.get_span(), TyKind::Err)
        })
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

    crate fn mk_id(&self) -> NodeId {
        let id = self.id_counter.get();
        self.id_counter.set(id + 1);
        NodeId::new(id)
    }

    crate fn try_parse<R, P>(&mut self, parser: &mut P) -> Option<R>
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

    crate fn mk_path(&self, span: Span, segments: Vec<PathSegment>) -> Path {
        Path { id: self.mk_id(), span, segments }
    }

    crate fn mk_expr(&self, span: Span, kind: ExprKind) -> P<Expr> {
        box Expr { span, id: self.mk_id(), kind }
    }

    crate fn mk_infer_ty(&self) -> P<Ty> {
        self.mk_ty(self.empty_span(), TyKind::Infer)
    }

    crate fn mk_ty(&self, span: Span, kind: TyKind) -> P<Ty> {
        box Ty { span, id: self.mk_id(), kind }
    }

    crate fn mk_pat(&self, span: Span, kind: PatternKind) -> P<Pattern> {
        if let PatternKind::Ident(ident, _, _) = kind {
            if !ident.is_lower() {
                self.build_err(span, ParseError::ExpectLowercaseIdentifier(*ident)).emit();
            }
        }
        box Pattern { span, id: self.mk_id(), kind }
    }

    crate fn mk_stmt(&self, span: Span, kind: StmtKind) -> P<Stmt> {
        box Stmt { span, id: self.mk_id(), kind }
    }

    crate fn mk_item(&self, span: Span, vis: Visibility, ident: Ident, kind: ItemKind) -> P<Item> {
        match kind {
            ItemKind::Fn(..) if !ident.is_lower() =>
                self.build_err(span, ParseError::ExpectLowercaseIdentifier(*ident)).emit(),
            ItemKind::Enum(..) | ItemKind::Struct(..) if !ident.is_upper() =>
                self.build_err(span, ParseError::ExpectUppercaseIdentifier(*ident)).emit(),
            _ => {}
        }
        box Item { span, id: self.mk_id(), ident, vis, kind }
    }

    // same as next except the return value is suppressed
    crate fn bump(&mut self) {
        self.next();
    }

    crate fn next(&mut self) -> Tok {
        let tok = self.peek();
        self.idx += 1;
        tok
    }

    crate fn reached_eof(&self) -> bool {
        self.tokens[self.idx].ttype == TokenType::Eof
    }

    crate fn safe_peek(&self) -> ParseResult<'a, Tok> {
        if !self.reached_eof() {
            Ok(self.tokens[self.idx])
        } else {
            Err(self.build_err(self.empty_span(), ParseError::Eof))
        }
    }

    crate fn peek(&self) -> Tok {
        self.safe_peek().ok().unwrap()
    }

    crate fn prev(&self) -> Tok {
        self.tokens[self.idx - 1]
    }

    crate fn accept_literal(&mut self) -> Option<(LiteralKind, Span)> {
        let Tok { span, ttype } = self.safe_peek().ok()?;
        match ttype {
            TokenType::Literal { kind, .. } => {
                self.idx += 1;
                Some((kind, span))
            }
            _ => None,
        }
    }

    /// expect identifier starting with uppercase letter
    /// see comments on `expect_lident` on return values
    crate fn expect_uident(&mut self) -> ParseResult<'a, Ident> {
        let ident = self.expect_ident()?;
        if !ident.is_upper() {
            self.build_err(ident.span, ParseError::ExpectUppercaseIdentifier(*ident)).emit();
        }
        Ok(ident)
    }

    /// expect lowercase identifier (starts with lowercase letter or _)
    /// we return an `Ok` as we can still continue parsing, but we report an error immediately
    crate fn expect_lident(&mut self) -> ParseResult<'a, Ident> {
        let ident = self.expect_ident()?;
        if !ident.is_lower() {
            self.build_err(ident.span, ParseError::ExpectLowercaseIdentifier(*ident)).emit();
        }
        Ok(ident)
    }

    // use only for path segments where both lidents and uidents are valid
    crate fn expect_ident(&mut self) -> ParseResult<'a, Ident> {
        let token = self.safe_peek()?;
        let Tok { span, ttype } = token;
        match ttype {
            TokenType::Ident(symbol) => {
                self.idx += 1;
                Ok(Ident { span, symbol })
            }
            _ => Err(self
                .build_err(token.span, ParseError::Expected(TokenType::Ident(kw::Empty), token))),
        }
    }

    crate fn accept_lident(&mut self) -> Option<Ident> {
        self.expect_lident().ok()
    }

    crate fn accept(&mut self, ttype: TokenType) -> Option<Tok> {
        self.expect(ttype).ok()
    }

    crate fn accept_one_of<'i, I>(&mut self, ttypes: &'i I) -> Option<Tok>
    where
        &'i I: IntoIterator<Item = &'i TokenType>,
    {
        ttypes.into_iter().fold(None, |acc, &t| acc.or_else(|| self.accept(t)))
    }

    crate fn expect(&mut self, ttype: TokenType) -> ParseResult<'a, Tok> {
        let t = self.safe_peek()?;
        if t.ttype == ttype {
            self.idx += 1;
            Ok(t)
        } else {
            Err(self.build_err(t.span, ParseError::Expected(ttype, t)))
        }
    }

    crate fn expect_one_of<'i, I>(&mut self, ttypes: &'i I) -> ParseResult<'a, Tok>
    where
        &'i I: IntoIterator<Item = &'i TokenType>,
    {
        self.accept_one_of(ttypes).ok_or_else(|| {
            let tok = self.peek();
            let err = ParseError::ExpectedOneOf(ttypes.into_iter().cloned().collect(), tok);
            self.build_err(tok.span, err)
        })
    }
}

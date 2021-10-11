use super::*;
use codespan::ByteIndex;
use lc_ast::*;
use lc_error::*;
use lc_index::Idx;
use lc_lex::*;
use lc_session::Session;
use lc_span::{self, kw, FileIdx, Span, SpanIdx, Symbol, ROOT_FILE_IDX};
use std::cell::Cell;
use std::error::Error;
use std::ops::{Deref, DerefMut};

pub struct Parser<'a> {
    pub sess: &'a Session,
    fparser: Option<FileParser>,
    id_counter: Cell<usize>,
}

/// parser for a single source file
pub struct FileParser {
    pub(crate) file: FileIdx,
    tokens: Vec<Token>,
    idx: usize,
}

impl FileParser {
    pub fn new(file: FileIdx) -> Self {
        let tokens = Lexer::new().lex(file).into_iter().collect();
        Self { file, tokens, idx: 0 }
    }
}

impl<'a> Parser<'a> {
    pub fn new(sess: &'a Session) -> Self {
        Self { fparser: None, id_counter: Cell::new(0), sess }
    }

    /// entry point to parsing; parses starting from root file
    pub fn parse(&mut self) -> Option<Ast> {
        self.with_file(ROOT_FILE_IDX, |parser| {
            let ast = AstParser.parse(parser).map_err(|err| err.emit()).ok()?;
            validate::AstValidator::default().visit_ast(&ast);
            Some(ast)
        })
    }

    pub(crate) fn with_file<R>(&mut self, file: FileIdx, f: impl FnOnce(&mut Self) -> R) -> R {
        let fparser = self.fparser.take();
        self.fparser = Some(FileParser::new(file));
        let ret = f(self);
        self.fparser = fparser;
        ret
    }

    pub fn dump_token_stream(&self) {
        for token in &self.tokens[self.idx..] {
            eprintln!("{:?}", token.kind);
        }
    }

    /// directly moves the parser's cursor backwards by `u` steps
    /// avoid using this unless necessary
    pub fn backtrack(&mut self, u: usize) {
        self.idx -= u;
    }

    pub fn mk_span(&self, start: impl SpanIdx, end: impl SpanIdx) -> Span {
        Span::new(self.file, start, end)
    }

    pub fn build_err(&self, span: impl Into<MultiSpan>, err: impl Error) -> DiagnosticBuilder<'a> {
        self.sess.build_error(span, err)
    }

    pub fn parse_mutability(&mut self) -> Mutability {
        match self.accept(TokenKind::Mut) {
            Some(_) => Mutability::Mut,
            None => Mutability::Imm,
        }
    }

    pub(crate) fn within_braces<R>(
        &mut self,
        mut parser: impl Parse<'a, Output = R>,
    ) -> ParseResult<'a, R> {
        self.expect(TokenKind::OpenBrace)?;
        let r = parser.parse(self)?;
        self.expect(TokenKind::CloseBrace)?;
        Ok(r)
    }

    pub(crate) fn within_parens<R>(
        &mut self,
        mut parser: impl Parse<'a, Output = R>,
    ) -> ParseResult<'a, R> {
        self.expect(TokenKind::OpenParen)?;
        let r = parser.parse(self)?;
        self.expect(TokenKind::CloseParen)?;
        Ok(r)
    }

    /// runs some parser and returns the result and the span that it consumed
    /// `include_prev` indicates whether the previous token is to be included in the span or not
    pub(crate) fn with_span<R>(
        &mut self,
        mut parser: impl Parse<'a, Output = R>,
        include_prev: bool,
    ) -> ParseResult<'a, (Span, R)> {
        let lo = if include_prev {
            self.tokens[self.idx - 1].span.start()
        } else {
            self.curr_span_start()
        };
        let p = parser.parse(self)?;
        Ok((self.mk_span(lo, self.prev_span_end()), p))
    }

    /// returns true if the current token is an ident
    /// similar to `accept_ident` except the token stream is not advanced
    pub fn is_ident(&self) -> ParseResult<'a, Option<Ident>> {
        let tok = self.safe_peek()?;
        Ok(if let TokenKind::Ident(symbol) = tok.kind {
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

        let s = match token.kind {
            TokenKind::Literal { kind: LiteralKind::Float { .. }, suffix_start: _ } =>
                span.to_string(),
            _ => unreachable!(),
        };
        let idx = s.find('.').unwrap();
        let x = Symbol::intern(&s[..idx]);
        let xspan = self.mk_span(span.start(), span.start().to_usize() + idx);
        let y = Symbol::intern(&s[idx + 1..]);
        let yspan = self.mk_span(span.start().to_usize() + idx + 1, span.end());
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
        self.mk_span(idx, idx)
    }

    pub fn parse_stmt(&mut self) -> ParseResult<'a, P<Stmt>> {
        StmtParser.parse(self)
    }

    pub fn parse_item(&mut self) -> ParseResult<'a, P<Item>> {
        ItemParser.parse(self)
    }

    /// entry point to parsing a single expression from a file
    /// used for testing purposes
    pub fn test_parse_expr(&mut self) -> P<Expr> {
        self.with_file(ROOT_FILE_IDX, |parser| parser.parse_expr())
    }

    pub fn test_parse_tt(&mut self) -> TokenStream {
        let tokens = Lexer::new().lex(ROOT_FILE_IDX);
        TokenTreeParser::new(self.sess, tokens).parse_token_stream()
    }

    pub fn test_parse_macro(&mut self) -> ParseResult<Macro> {
        self.with_file(ROOT_FILE_IDX, |parser| parser.parse_macro())
    }

    pub fn parse_expr(&mut self) -> P<Expr> {
        ExprParser.parse(self).unwrap_or_else(|err| {
            err.emit();
            self.mk_expr(err.get_first_span(), ExprKind::Err)
        })
    }

    pub fn parse_macro(&mut self) -> ParseResult<'a, Macro> {
        MacroParser.parse(self)
    }

    pub fn parse_generics(&mut self) -> ParseResult<'a, Generics> {
        GenericsParser.parse(self)
    }

    pub fn parse_block(&mut self, open_brace: Token) -> ParseResult<'a, P<Block>> {
        BlockParser { open_brace, is_unsafe: false }.parse(self)
    }

    pub fn parse_ty(&mut self, allow_infer: bool) -> P<Ty> {
        TyParser { allow_infer }.parse(self).unwrap_or_else(|err| {
            err.emit();
            self.mk_ty(err.get_first_span(), TyKind::Err)
        })
    }

    pub fn parse_pattern(&mut self) -> ParseResult<'a, P<Pattern>> {
        PatParser.parse(self)
    }

    pub fn parse_module_path(&mut self) -> ParseResult<'a, Path> {
        PathParser { kind: PathKind::Module }.parse(self)
    }

    pub fn parse_type_path(&mut self) -> ParseResult<'a, Path> {
        PathParser { kind: PathKind::Type }.parse(self)
    }

    pub fn parse_expr_path(&mut self) -> ParseResult<'a, Path> {
        PathParser { kind: PathKind::Expr }.parse(self)
    }

    pub(crate) fn mk_id(&self) -> NodeId {
        let id = self.id_counter.get();
        self.id_counter.set(id + 1);
        NodeId::new(id)
    }

    pub(crate) fn try_parse<R, P>(&mut self, parser: &mut P) -> Option<R>
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

    pub(crate) fn mk_path(&self, span: Span, segments: Vec<PathSegment>) -> Path {
        Path { id: self.mk_id(), span, segments }
    }

    pub(crate) fn mk_expr(&self, span: Span, kind: ExprKind) -> P<Expr> {
        box Expr { span, id: self.mk_id(), kind }
    }

    pub(crate) fn mk_infer_ty(&self) -> P<Ty> {
        self.mk_ty(self.empty_span(), TyKind::Infer)
    }

    pub(crate) fn mk_ty_err(&self, span: Span) -> P<Ty> {
        self.mk_ty(span, TyKind::Err)
    }

    pub(crate) fn mk_ty(&self, span: Span, kind: TyKind) -> P<Ty> {
        box Ty { span, id: self.mk_id(), kind }
    }

    pub(crate) fn mk_pat(&self, span: Span, kind: PatternKind) -> P<Pattern> {
        if let PatternKind::Ident(ident, _, _) = kind {
            if !ident.is_lower() {
                self.build_err(span, ParseError::ExpectLowercaseIdentifier(*ident)).emit();
            }
        }
        box Pattern { span, id: self.mk_id(), kind }
    }

    pub(crate) fn mk_stmt(&self, span: Span, kind: StmtKind) -> P<Stmt> {
        box Stmt { span, id: self.mk_id(), kind }
    }

    pub(crate) fn mk_item(
        &self,
        span: Span,
        vis: Visibility,
        ident: Ident,
        kind: ItemKind,
    ) -> P<Item> {
        // validates identifier and visibility
        match kind {
            ItemKind::Fn(..) if !ident.is_lower() =>
                self.build_err(span, ParseError::ExpectLowercaseIdentifier(*ident)).emit(),
            ItemKind::Enum(..) | ItemKind::Struct(..) if !ident.is_upper() =>
                self.build_err(span, ParseError::ExpectUppercaseIdentifier(*ident)).emit(),
            _ => {}
        }

        match kind {
            ItemKind::Extern(..) | ItemKind::Impl { .. } =>
                if *vis == VisibilityKind::Public {
                    self.build_err(span, ParseError::RedundantVisibilityModifier).emit()
                },
            _ => {}
        }

        box Item { span, id: self.mk_id(), ident, vis, kind }
    }

    // same as next except the return value is suppressed
    pub(crate) fn bump(&mut self) {
        self.next();
    }

    pub(crate) fn next(&mut self) -> Token {
        let tok = self.peek();
        self.idx += 1;
        tok
    }

    pub(crate) fn safe_next(&mut self) -> ParseResult<'a, Token> {
        let tok = self.safe_peek();
        self.idx += 1;
        tok
    }

    pub(crate) fn reached_eof(&self) -> bool {
        self.tokens[self.idx].kind == TokenKind::Eof
    }

    pub(crate) fn safe_peek(&self) -> ParseResult<'a, Token> {
        if !self.reached_eof() {
            Ok(self.tokens[self.idx])
        } else {
            Err(self.build_err(self.empty_span(), ParseError::Eof))
        }
    }

    pub(crate) fn peek(&self) -> Token {
        self.safe_peek().ok().unwrap()
    }

    pub(crate) fn prev(&self) -> Token {
        self.tokens[self.idx - 1]
    }

    pub(crate) fn accept_literal(&mut self) -> Option<(LiteralKind, Span)> {
        self.expect_literal().ok()
    }

    pub(crate) fn expect_literal(&mut self) -> ParseResult<'a, (LiteralKind, Span)> {
        let Token { span, kind: ttype } = self.safe_peek()?;
        match ttype {
            TokenKind::Literal { kind, .. } => {
                self.idx += 1;
                Ok((kind, span))
            }
            _ => Err(self.build_err(span, ParseError::ExpectedLiteral(ttype))),
        }
    }

    /// expect identifier starting with uppercase letter
    /// see comments on `expect_lident` on return values
    pub(crate) fn expect_uident(&mut self) -> ParseResult<'a, Ident> {
        let ident = self.expect_ident()?;
        if !ident.is_upper() {
            self.build_err(ident.span, ParseError::ExpectUppercaseIdentifier(*ident)).emit();
        }
        Ok(ident)
    }

    /// expect lowercase identifier (starts with lowercase letter or _)
    /// we return an `Ok` as we can still continue parsing, but we report an error immediately
    pub(crate) fn expect_lident(&mut self) -> ParseResult<'a, Ident> {
        let ident = self.expect_ident()?;
        if !ident.is_lower() {
            self.build_err(ident.span, ParseError::ExpectLowercaseIdentifier(*ident)).emit();
        }
        Ok(ident)
    }

    // use only for path segments where both lidents and uidents are valid
    pub(crate) fn expect_ident(&mut self) -> ParseResult<'a, Ident> {
        let token = self.safe_peek()?;
        let Token { span, kind: ttype } = token;
        match ttype {
            TokenKind::Ident(symbol) => {
                self.idx += 1;
                Ok(Ident { span, symbol })
            }
            _ => Err(self
                .build_err(token.span, ParseError::Expected(TokenKind::Ident(kw::Empty), token))),
        }
    }

    pub(crate) fn accept_lident(&mut self) -> Option<Ident> {
        self.expect_lident().ok()
    }

    pub(crate) fn accept(&mut self, ttype: TokenKind) -> Option<Token> {
        self.expect(ttype).ok()
    }

    pub(crate) fn parse_tt_group(&mut self) -> TokenGroup {
        // HACK abusing the fact that [TokenIterator] is current of type [std::vec::IntoIter]
        let group = TokenTreeParser::new(self.sess, self.tokens[self.idx..].to_vec().into_iter())
            .parse_token_group();
        self.idx += group.size();
        group
    }

    pub(crate) fn accept_one_of(
        &mut self,
        ttypes: impl IntoIterator<Item = TokenKind>,
    ) -> Option<Token> {
        ttypes.into_iter().fold(None, |acc, t| acc.or_else(|| self.accept(t)))
    }

    pub(crate) fn parse_abi(&mut self) -> ParseResult<'a, Abi> {
        let symbol = self.accept_str();
        let symbol = match symbol {
            Some(symbol) => symbol,
            None => return Ok(Abi::L),
        };
        match symbol.as_str() {
            "l-intrinsic" => Ok(Abi::Intrinsic),
            abi => Err(self.build_err(symbol.span, ParseError::InvalidAbi(abi.to_owned()))),
        }
    }

    pub(crate) fn accept_str(&mut self) -> Option<Ident> {
        self.expect_str().ok()
    }

    pub(crate) fn expect_str(&mut self) -> ParseResult<'a, Ident> {
        let (kind, span) = self.expect_literal()?;
        match kind {
            LiteralKind::Str { terminated } => {
                if !terminated {
                    return Err(self.build_err(span, ParseError::UnterminatedStringLiteral));
                }
                // the span includes the surrounding quotes, so we just chop them off
                let symbol = span.with_slice(|slice| Symbol::intern(&slice[1..slice.len() - 1]));
                Ok(Ident::new(span, symbol))
            }
            _ => todo!(),
        }
    }

    pub(crate) fn expect(&mut self, ttype: TokenKind) -> ParseResult<'a, Token> {
        let t = self.safe_peek()?;
        if t.kind == ttype {
            self.idx += 1;
            Ok(t)
        } else {
            Err(self.build_err(t.span, ParseError::Expected(ttype, t)))
        }
    }

    pub(crate) fn expect_one_of(
        &mut self,
        tkinds: impl IntoIterator<Item = TokenKind> + Copy,
    ) -> ParseResult<'a, Token> {
        self.accept_one_of(tkinds).ok_or_else(|| {
            let tok = self.peek();
            let err = ParseError::ExpectedOneOf(tkinds.into_iter().collect(), tok);
            self.build_err(tok.span, err)
        })
    }
}

impl<'a> Deref for Parser<'a> {
    type Target = FileParser;

    fn deref(&self) -> &Self::Target {
        self.fparser.as_ref().unwrap()
    }
}

impl<'a> DerefMut for Parser<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.fparser.as_mut().unwrap()
    }
}

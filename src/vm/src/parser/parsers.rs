//! general use parsers

use super::*;
use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::lexer::{Tok, TokenType};
use crate::span::Span;

pub struct RetParser {
    pub ret_kw: Tok,
}

impl Parse for RetParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let expr = parser.try_parse(&mut ExprParser);
        let span = self
            .ret_kw
            .span
            .merge(expr.as_ref().map(|expr| expr.span).unwrap_or_else(|| parser.empty_span()));
        Ok(parser.mk_expr(span, ExprKind::Ret(expr)))
    }
}

pub struct FnSigParser {
    pub require_type_annotations: bool,
}

impl Parse for FnSigParser {
    type Output = FnSig;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let require_type_annotations = self.require_type_annotations;
        parser.expect(TokenType::OpenParen)?;
        let mut param_parser = PunctuatedParser {
            inner: ParamParser { require_type_annotations },
            separator: TokenType::Comma,
        };
        let inputs = param_parser.parse(parser)?;
        parser.expect(TokenType::CloseParen)?;
        let mut output =
            parser.accept(TokenType::RArrow).map(|_arrow| TyParser.parse(parser)).transpose()?;

        if output.is_none() && !require_type_annotations {
            output = Some(parser.mk_infer_ty())
        }

        Ok(FnSig { params: inputs, ret_ty: output })
    }
}

/// parses function parameter <pat> (: <ty>)?
pub struct ParamParser {
    pub require_type_annotations: bool,
}

impl Parse for ParamParser {
    type Output = Param;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let pattern = PatParser.parse(parser)?;
        let ty = if let Some(_colon) = parser.accept(TokenType::Colon) {
            TyParser.parse(parser)
        } else if self.require_type_annotations {
            Err(ParseError::require_type_annotations(pattern.span))
        } else {
            Ok(parser.mk_infer_ty())
        }?;
        Ok(Param { span: pattern.span.merge(ty.span), id: parser.mk_id(), pattern, ty })
    }
}

pub struct VisibilityParser;

impl Parse for VisibilityParser {
    type Output = Visibility;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(pub_keyword) = parser.accept(TokenType::Pub) {
            Ok(Visibility { span: pub_keyword.span, node: VisibilityKind::Public })
        } else {
            Ok(Visibility { span: parser.empty_span(), node: VisibilityKind::Private })
        }
    }
}

/// implement Parser for TokenType to be used as a separator
impl Parse for TokenType {
    type Output = Tok;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        parser.expect(*self)
    }
}

/// parses a given parser zero or more times punctuated by some given separator parser
/// this parser accepts a trailing separator
///
/// <punctuated> = Ɛ | <inner> ( <sep> <inner> )* <sep>?
pub struct PunctuatedParser<P, S> {
    pub inner: P,
    pub separator: S,
}

impl<P, S> Parse for PunctuatedParser<P, S>
where
    P: Parse,
    S: Parse,
{
    type Output = Vec<P::Output>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut vec = vec![];
        // if the first parse already fails then just return empty vector
        let p = match self.inner.parse(parser) {
            Ok(p) => p,
            Err(_) => return Ok(vec),
        };
        vec.push(p);
        while self.separator.parse(parser).is_ok() {
            match self.inner.parse(parser) {
                Ok(p) => vec.push(p),
                Err(_) => break,
            }
        }
        // parse the trailing separator if there is one
        let _ = self.separator.parse(parser);
        Ok(vec)
    }
}

/// similar to `PunctuatedParser` except parses one or more occurences of `inner`
/// accepts trailing separator
pub struct Punctuated1Parser<P, S> {
    pub inner: P,
    pub separator: S,
}

impl<P, S> Parse for Punctuated1Parser<P, S>
where
    P: Parse,
    S: Parse,
{
    type Output = Vec<P::Output>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut vec = vec![self.inner.parse(parser)?];
        while self.separator.parse(parser).is_ok() {
            vec.push(self.inner.parse(parser)?);
        }
        let _ = self.separator.parse(parser);
        Ok(vec)
    }
}

/// similar to `PunctuatedParser` except a single element tuple must have a trailing comma (to
/// differentiate it from a parenthesization)
/// <tuple> = () | '(' ( <inner> , )+ <inner>? ')'
pub struct TupleParser<P> {
    pub inner: P,
}

impl<P> Parse for TupleParser<P>
where
    P: Parse,
    P::Output: std::fmt::Debug,
{
    type Output = Vec<P::Output>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut vec = vec![];

        if parser.accept(TokenType::CloseParen).is_some() {
            return Ok(vec);
        }

        while parser.accept(TokenType::CloseParen).is_none() {
            vec.push(self.inner.parse(parser)?);
            if parser.accept(TokenType::Comma).is_none() {
                parser.expect(TokenType::CloseParen)?;
                break;
            }
        }
        Ok(vec)
    }
}

/// parser some inner parser within parentheses
pub struct ParenParser<P> {
    pub inner: P,
}

impl<P> Parse for ParenParser<P>
where
    P: Parse,
{
    type Output = P::Output;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let p = self.inner.parse(parser)?;
        parser.expect(TokenType::CloseParen)?;
        Ok(p)
    }
}

pub struct StructExprParser {
    path: Path,
}

impl Parse for StructExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let field_parser = |parser: &mut Parser| {
            let ident = parser.expect_ident()?;
            let expr = if parser.accept(TokenType::Colon).is_some() {
                ExprParser.parse(parser)?
            } else {
                let span = parser.empty_span();
                // construct a Path node which the a single segment with ident `ident`
                // this is the implementation of the struct shorthand
                // S { t } -> S { t: t }
                let segment = PathSegment { id: parser.mk_id(), ident, args: None };
                let path = Path { span, segments: vec![segment] };
                parser.mk_expr(span, ExprKind::Path(path))
            };
            let span = ident.span.merge(expr.span);
            Ok(Field { ident, expr, span, id: parser.mk_id() })
        };
        let (span, fields) = PunctuatedParser { inner: field_parser, separator: TokenType::Comma }
            .spanned(false)
            .parse(parser)?;
        parser.expect(TokenType::CloseBrace)?;
        let path = std::mem::take(&mut self.path);
        Ok(parser.mk_expr(span, ExprKind::Struct(path, fields)))
    }
}

pub struct PathExprParser;

impl Parse for PathExprParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let (span, path) = PathParser.spanned(false).parse(parser)?;
        // if the path is immediately followed by an open brace, it is a struct expr
        // SomeStruct {
        //    x: int,
        //    y: bool,
        // }
        if let Some(_) = parser.accept(TokenType::OpenBrace) {
            StructExprParser { path }.parse(parser)
        } else {
            Ok(parser.mk_expr(span, ExprKind::Path(path)))
        }
    }
}

/// parses a path a::b::c
pub struct PathParser;

impl Parse for PathParser {
    type Output = Path;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let separator = |parser: &mut Parser| {
            parser.expect(TokenType::Colon)?;
            parser.expect(TokenType::Colon)
        };
        let (span, segments) = Punctuated1Parser { inner: PathSegmentParser, separator }
            .spanned(false)
            .parse(parser)?;
        // if the path is immediately followed by an open brace, it could be a struct expr
        Ok(Path { span, segments })
    }
}

pub struct PathSegmentParser;

impl Parse for PathSegmentParser {
    type Output = PathSegment;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let ident = parser.expect_ident()?;
        // with the generics of the initial ident
        Ok(PathSegment { ident, id: parser.mk_id(), args: None })
    }
}

pub struct BlockParser {
    pub open_brace: Tok,
}

impl Parse for BlockParser {
    type Output = P<Block>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut stmts = vec![];
        let close_brace = loop {
            if let Some(close_brace) = parser.accept(TokenType::CloseBrace) {
                break close_brace;
            }
            stmts.push(StmtParser.parse(parser)?);
        };
        let span = self.open_brace.span.merge(close_brace.span);
        if !stmts.is_empty() {
            let len = stmts.len() - 1;
            // check there are no missing semicolons in expression statements
            for stmt in &stmts[..len] {
                if let StmtKind::Expr(_) = stmt.kind {
                    return Err(ParseError::expected_semi(stmt.span));
                }
            }
            // for easier typechecking when the final statement is diverging
            stmts[len].upgrade_diverging_to_expr();
        }
        Ok(box Block { span, id: parser.mk_id(), stmts })
    }
}

/// fn <ident>? (<pat>:<ty>) -> ret => <body>
/// "=>" is optional if body is a block expression
/// xs.map(fn(x) => y);
pub struct ClosureParser {
    pub fn_kw: Tok,
}

impl Parse for ClosureParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let name = parser.accept_ident();
        let sig = FnSigParser { require_type_annotations: false }.parse(parser)?;
        let body = if let Some(open_brace) = parser.accept(TokenType::OpenBrace) {
            let block = BlockParser { open_brace }.parse(parser)?;
            parser.mk_expr(block.span, ExprKind::Block(block))
        } else {
            parser.expect(TokenType::RFArrow)?;
            ExprParser.parse(parser)?
        };
        let span = self.fn_kw.span.merge(body.span);
        Ok(parser.mk_expr(span, ExprKind::Closure(name, sig, body)))
    }
}

pub struct IfParser {
    pub if_kw: Tok,
}

impl Parse for IfParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let cond = ExprParser.parse(parser)?;
        let open_brace = parser.expect(TokenType::OpenBrace)?;
        let thn = BlockParser { open_brace }.parse(parser)?;
        let els = if let Some(else_kw) = parser.accept(TokenType::Else) {
            Some(ElseParser { else_kw }.parse(parser)?)
        } else {
            None
        };
        let span = self.if_kw.span.merge(parser.empty_span());
        Ok(parser.mk_expr(span, ExprKind::If(cond, thn, els)))
    }
}

pub struct ElseParser {
    pub else_kw: Tok,
}

impl Parse for ElseParser {
    type Output = P<Expr>;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        if let Some(if_kw) = parser.accept(TokenType::If) {
            IfParser { if_kw }.parse(parser)
        } else if let Some(open_brace) = parser.accept(TokenType::OpenBrace) {
            let (span, block) = BlockParser { open_brace }.spanned(true).parse(parser)?;
            Ok(parser.mk_expr(span, ExprKind::Block(block)))
        } else {
            Err(ParseError::unimpl())
        }
    }
}

pub struct GenericsParser;

impl Parse for GenericsParser {
    type Output = Generics;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let mut span = parser.empty_span();
        let params = if parser.accept(TokenType::Lt).is_some() {
            let params = PunctuatedParser { inner: TyParamParser, separator: TokenType::Comma }
                .parse(parser)?;
            let gt = parser.expect(TokenType::Gt)?;
            span = span.merge(gt.span);
            params
        } else {
            vec![]
        };
        Ok(Generics { params, span })
    }
}

pub struct TyParamParser;

impl Parse for TyParamParser {
    type Output = TyParam;

    fn parse(&mut self, parser: &mut Parser) -> ParseResult<Self::Output> {
        let ident = parser.expect_ident()?;
        let default = parser.accept(TokenType::Eq).map(|_| TyParser.parse(parser)).transpose()?;
        // eventually parse bounds here
        Ok(TyParam { span: ident.span, id: parser.mk_id(), ident, default })
    }
}

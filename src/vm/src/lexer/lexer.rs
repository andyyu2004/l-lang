use super::{lexing, LiteralKind, Symbol};
use crate::span::{self, Span};
use itertools::Itertools;
use lazy_static::lazy_static;
use lexing::TokenKind;
use maplit::hashmap;
use once_cell::unsync::Lazy;
use std::collections::HashMap;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = hashmap! {
        "false" => TokenType::False,
        "true" => TokenType::True,
        "fn" => TokenType::Fn,
        "pub" => TokenType::Pub,
        "enum" => TokenType::Enum,
        "struct" => TokenType::Struct,
        "let" => TokenType::Let,
        "if" => TokenType::If,
        "else" => TokenType::Else,
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
crate struct Tok {
    pub span: Span,
    pub ttype: TokenType,
}

crate struct Lexer<'ctx> {
    ctx: &'ctx mut span::Ctx,
}

impl<'ctx> Lexer<'ctx> {
    pub fn new(ctx: &'ctx mut span::Ctx) -> Self {
        Self { ctx }
    }

    /// transforms the rustc token with len into one with span
    pub fn lex(&mut self) -> Vec<Tok> {
        let src = self.ctx.main_file().src.to_owned();
        let mut span_index = 0;

        // note: it is important to filter after so that the spans are correct
        let tokens = lexing::tokenize(&src).collect_vec();
        let mut i = 0;
        let mut vec = vec![];

        while i < tokens.len() {
            let t = tokens[i];
            i += 1;
            let token = {
                let span = Span::new(span_index, span_index + t.len);
                span_index += t.len;
                let slice = &src[span.lo..span.hi];
                let kind = match t.kind {
                    TokenKind::Whitespace => continue,
                    TokenKind::Eq =>
                        if tokens[i].kind == TokenKind::Gt {
                            i += 1;
                            span_index += 1;
                            TokenType::RFArrow
                        } else {
                            TokenType::Eq
                        },
                    TokenKind::Minus =>
                        if tokens[i].kind == TokenKind::Gt {
                            i += 1;
                            span_index += 1;
                            TokenType::RArrow
                        } else {
                            TokenType::Minus
                        },

                    TokenKind::LineComment => continue,
                    TokenKind::BlockComment { terminated } => {
                        if !terminated {
                            panic!("unterminated block comment")
                        }
                        continue;
                    }
                    TokenKind::Ident =>
                        if let Some(&keyword) = KEYWORDS.get(slice) {
                            keyword
                        } else {
                            let symbol = self.ctx.symbol_interner.intern(slice);
                            TokenType::Ident(symbol)
                        },

                    TokenKind::RawIdent => todo!(),
                    TokenKind::Literal { kind, suffix_start } =>
                        TokenType::Literal { kind, suffix_start },
                    TokenKind::Lifetime { starts_with_number } =>
                        todo!("maybe use lifetime syntax as generic parameter (like ocaml)"),
                    TokenKind::Semi => TokenType::Semi,
                    TokenKind::Underscore => TokenType::Underscore,
                    TokenKind::Comma => TokenType::Comma,
                    TokenKind::Dot => TokenType::Dot,
                    TokenKind::OpenParen => TokenType::OpenParen,
                    TokenKind::CloseParen => TokenType::CloseParen,
                    TokenKind::OpenBrace => TokenType::OpenBrace,
                    TokenKind::CloseBrace => TokenType::CloseBrace,
                    TokenKind::OpenBracket => TokenType::OpenSqBracket,
                    TokenKind::CloseBracket => TokenType::CloseSqBracket,
                    TokenKind::At => TokenType::At,
                    TokenKind::Pound => TokenType::Pound,
                    TokenKind::Tilde => TokenType::Tilde,
                    TokenKind::Question => TokenType::Question,
                    TokenKind::Colon => TokenType::Colon,
                    TokenKind::Dollar => TokenType::Dollar,
                    TokenKind::Not => TokenType::Not,
                    TokenKind::Lt => TokenType::Lt,
                    TokenKind::Gt => TokenType::Gt,
                    TokenKind::And => TokenType::And,
                    TokenKind::Or => TokenType::Or,
                    TokenKind::Plus => TokenType::Plus,
                    TokenKind::Star => TokenType::Star,
                    TokenKind::Slash => TokenType::Slash,
                    TokenKind::Caret => TokenType::Caret,
                    TokenKind::Percent => TokenType::Percent,
                    TokenKind::Eof => TokenType::Eof,
                    TokenKind::Unknown => TokenType::Unknown,
                };

                Tok { span, ttype: kind }
            };
            vec.push(token)
        }

        // just manually add a <eof> token for easier parsing
        vec.push(Tok { span: Span::new(span_index, span_index), ttype: TokenType::Eof });
        vec
    }
}

/// token kind that has been further processed to include keywords
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {
    /// ->
    RArrow,
    /// =>
    RFArrow,
    Pub,
    Else,
    Let,
    Struct,
    Enum,
    Fn,
    False,
    True,
    If,
    // Multi-char tokens:
    /// "// comment"
    LineComment,
    /// "/* block comment */"
    /// Block comments can be recursive, so the sequence like "/* /* */"
    /// will not be considered terminated and will result in a parsing error.
    BlockComment {
        terminated: bool,
    },
    /// Any whitespace characters sequence.
    Whitespace,
    /// "ident" or "continue"
    /// "r#ident"
    RawIdent,
    /// "12_u8", "1.0e-40", "b"123"". See `LiteralKind` for more details.
    Literal {
        kind: LiteralKind,
        suffix_start: usize,
    },
    Lifetime {
        starts_with_number: bool,
    },

    // One-char tokens:
    /// ";"
    Semi,
    /// ","
    Comma,
    /// "."
    Dot,
    /// "("
    OpenParen,
    /// ")"
    CloseParen,
    /// "{"
    OpenBrace,
    /// "}"
    CloseBrace,
    /// "["
    OpenSqBracket,
    /// "]"
    CloseSqBracket,
    /// "@"
    At,
    /// "#"
    Pound,
    /// "~"
    Tilde,
    /// "?"
    Question,
    /// ":"
    Colon,
    /// "$"
    Dollar,
    /// "="
    Eq,
    /// "!"
    Not,
    /// "<"
    Lt,
    /// ">"
    Gt,
    /// "-"
    Minus,
    /// "&"
    And,
    /// "|"
    Or,
    /// "+"
    Plus,
    /// "*"
    Star,
    /// "/"
    Slash,
    /// "^"
    Caret,
    /// "%"
    Percent,
    Eof,
    /// Unknown token, not expected by the lexer, e.g. "№"
    Unknown,
    /// "_"
    Underscore,
    Ident(Symbol),
}

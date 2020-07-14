use super::{lexing, LiteralKind, Symbol};
use crate::span::{self, Span};
use itertools::Itertools;
use lexing::TokenKind;
use maplit::hashmap;
use once_cell::unsync::Lazy;
use std::collections::HashMap;

const KEYWORDS: Lazy<HashMap<&'static str, TokenType>> = Lazy::new(|| {
    hashmap! {
        "false" => TokenType::False,
        "true" => TokenType::True,
        "fn" => TokenType::Fn,
        "pub" => TokenType::Pub,
        "enum" => TokenType::Enum,
        "struct" => TokenType::Struct,
        "let" => TokenType::Let,
    }
});

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
        let mut i = 0;

        // note: it is important to filter after so that the spans are correct
        let mut tokens = lexing::tokenize(&src)
            .map(|t| {
                let span = Span::new(i, i + t.len);
                i += t.len;
                let slice = &src[span.lo..span.hi];
                let kind = match t.kind {
                    TokenKind::LineComment => TokenType::Whitespace,
                    TokenKind::BlockComment { terminated } => TokenType::Whitespace,
                    TokenKind::Whitespace => TokenType::Whitespace,
                    TokenKind::Ident => {
                        // by convention, uppercase idents are Types
                        if let Some(&keyword) = KEYWORDS.get(slice) {
                            keyword
                        } else {
                            let symbol = self.ctx.symbol_interner.intern(slice);
                            if slice.chars().next().unwrap().is_uppercase() {
                                TokenType::Type(symbol)
                            } else {
                                TokenType::Ident(symbol)
                            }
                        }
                    }

                    TokenKind::RawIdent => todo!(),
                    TokenKind::Literal { kind, suffix_start } => {
                        TokenType::Literal { kind, suffix_start }
                    }
                    TokenKind::Lifetime { starts_with_number } => {
                        todo!("maybe use lifetime syntax as generic parameter (like ocaml)")
                    }
                    TokenKind::Semi => TokenType::Semi,
                    TokenKind::Underscore => TokenType::Underscore,
                    TokenKind::Comma => TokenType::Comma,
                    TokenKind::Dot => TokenType::Dot,
                    TokenKind::OpenParen => TokenType::OpenParen,
                    TokenKind::CloseParen => TokenType::CloseParen,
                    TokenKind::OpenBrace => TokenType::OpenBrace,
                    TokenKind::CloseBrace => TokenType::CloseBrace,
                    TokenKind::OpenBracket => TokenType::OpenBracket,
                    TokenKind::CloseBracket => TokenType::CloseBracket,
                    TokenKind::At => TokenType::At,
                    TokenKind::Pound => TokenType::Pound,
                    TokenKind::Tilde => TokenType::Tilde,
                    TokenKind::Question => TokenType::Question,
                    TokenKind::Colon => TokenType::Colon,
                    TokenKind::Dollar => TokenType::Dollar,
                    TokenKind::Eq => TokenType::Eq,
                    TokenKind::Not => TokenType::Not,
                    TokenKind::Lt => TokenType::Lt,
                    TokenKind::Gt => TokenType::Gt,
                    TokenKind::Minus => TokenType::Minus,
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
            })
            .filter(|t| t.ttype != TokenType::Whitespace)
            .collect_vec();
        // just add a <eof> token for easier parsing
        tokens.push(Tok { span: Span::new(i, i), ttype: TokenType::Eof });
        tokens
    }
}

/// token kind that has been further processed to include keywords
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {
    Pub,
    Let,
    Struct,
    Enum,
    Fn,
    False,
    True,
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
    OpenBracket,
    /// "]"
    CloseBracket,
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
    Type(Symbol),
    Ident(Symbol),
}

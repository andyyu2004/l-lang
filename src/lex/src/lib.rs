mod lexing;

use itertools::Itertools;
use lazy_static::lazy_static;
use lexing::TokenKind;
pub use lexing::{Base, LiteralKind};
use maplit::hashmap;
use span::{self, with_interner, with_source_map, FileIdx, Span, Symbol};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::ops::Range;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = hashmap! {
        "fn" => TokenType::Fn,
        "macro" => TokenType::Macro,
        "box" => TokenType::Box,
        "trait" => TokenType::Trait,
        "break" => TokenType::Break,
        "continue" => TokenType::Continue,
        "match" => TokenType::Match,
        "internal" => TokenType::Internal,
        "mod" => TokenType::Mod,
        "false" => TokenType::False,
        "true" => TokenType::True,
        "pub" => TokenType::Pub,
        "enum" => TokenType::Enum,
        "struct" => TokenType::Struct,
        "let" => TokenType::Let,
        "if" => TokenType::If,
        "else" => TokenType::Else,
        "return" => TokenType::Return,
        "mut" => TokenType::Mut,
        "use" => TokenType::Use,
        "type" => TokenType::Type,
        "unsafe" => TokenType::Unsafe,
        "const" => TokenType::Const,
        "impl" => TokenType::Impl,
        "extern" => TokenType::Extern,
        "for" => TokenType::For,
        "loop" => TokenType::Loop,
        "while" => TokenType::While,
        "self" => TokenType::LSelf,
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Tok {
    pub span: Span,
    pub ttype: TokenType,
}

pub struct Lexer {}

impl Lexer {
    pub fn new() -> Self {
        Self {}
    }

    /// transforms the rustc token with len into one with span
    pub fn lex(&mut self, file: FileIdx) -> Vec<Tok> {
        with_source_map(|map| {
            let src: &str = map.get(file);
            let mut span_index = 0;

            // note: it is important to filter after so that the spans are correct
            let n = lexing::strip_shebang(&src).unwrap_or(0);
            let tokens = lexing::tokenize(&src[n..]).collect_vec();
            let mut i = 0;
            let mut vec = vec![];

            while i < tokens.len() {
                let t = tokens[i];
                i += 1;
                let token = {
                    let span = Span::new(file, span_index, span_index + t.len);
                    span_index += t.len;
                    let slice = &src[Range::from(*span)];
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
                        TokenKind::Colon =>
                            if tokens[i].kind == TokenKind::Colon {
                                i += 1;
                                span_index += 1;
                                TokenType::Dcolon
                            } else {
                                TokenType::Colon
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
                            } else if slice == "_" {
                                TokenType::Underscore
                            } else {
                                let symbol = with_interner(|interner| interner.intern(slice));
                                TokenType::Ident(symbol)
                            },
                        TokenKind::RawIdent => todo!(),
                        TokenKind::Literal { kind, suffix_start } =>
                            TokenType::Literal { kind, suffix_start },
                        TokenKind::Lifetime { .. } =>
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
                        TokenKind::Unknown => TokenType::Unknown,
                    };

                    Tok { span, ttype: kind }
                };
                vec.push(token)
            }

            // just manually add a <eof> token for easier parsing
            vec.push(Tok { span: Span::new(file, span_index, span_index), ttype: TokenType::Eof });
            vec
        })
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Ident(sym) => write!(f, "identifier `{}`", sym),
            _ => write!(f, "{:?}", self),
        }
    }
}
/// token kind that has been further processed to include keywords
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {
    Ident(Symbol),
    Break,
    Trait,
    Continue,
    While,
    Internal,
    Mod,
    Use,
    LSelf,
    Extern,
    Const,
    For,
    Loop,
    Impl,
    Unsafe,
    Match,
    Box,
    Type,
    /// ->
    RArrow,
    Mut,
    Return,
    /// =>
    RFArrow,
    Pub,
    Else,
    Let,
    Struct,
    Enum,
    Fn,
    Macro,
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
    /// "::"
    Dcolon,
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
}

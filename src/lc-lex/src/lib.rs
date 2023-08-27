#![feature(type_alias_impl_trait)]

mod lexing;
mod token_tree;

pub use lexing::{Base, LiteralKind};
pub use token_tree::*;

use itertools::Itertools;
use lazy_static::lazy_static;
use lc_span::{self, with_interner, with_source_map, FileIdx, Span, Symbol};
use lexing::RawTokenKind;
use maplit::hashmap;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::ops::Range;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenKind> = hashmap! {
        "fn" => TokenKind::Fn,
        "macro" => TokenKind::Macro,
        "box" => TokenKind::Box,
        "trait" => TokenKind::Trait,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "match" => TokenKind::Match,
        "internal" => TokenKind::Internal,
        "mod" => TokenKind::Mod,
        "false" => TokenKind::False,
        "true" => TokenKind::True,
        "pub" => TokenKind::Pub,
        "enum" => TokenKind::Enum,
        "struct" => TokenKind::Struct,
        "let" => TokenKind::Let,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "return" => TokenKind::Return,
        "mut" => TokenKind::Mut,
        "use" => TokenKind::Use,
        "type" => TokenKind::Type,
        "unsafe" => TokenKind::Unsafe,
        "const" => TokenKind::Const,
        "impl" => TokenKind::Impl,
        "extern" => TokenKind::Extern,
        "for" => TokenKind::For,
        "loop" => TokenKind::Loop,
        "while" => TokenKind::While,
        "self" => TokenKind::LSelf,
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {
    #[inline(always)]
    pub fn span(&self) -> Span {
        self.span
    }
}

pub struct Lexer {}

// pub type TokenIterator = impl Iterator<Item = Token>;
pub type TokenIterator = std::vec::IntoIter<Token>;

impl Lexer {
    pub fn new() -> Self {
        Self {}
    }

    /// transforms the rustc token with len into one with span
    // TODO this could do with an iterator implementation (just returning an iterator so callers are forced to use it correctly)
    pub fn lex(&mut self, file: FileIdx) -> TokenIterator {
        let src = with_source_map(|map| map.get(file).source());
        let mut span_index = 0;

        // note: it is important to filter after so that the spans are correct
        let n = lexing::strip_shebang(src).unwrap_or(0);
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
                    RawTokenKind::Whitespace => continue,
                    RawTokenKind::Eq =>
                        if tokens[i].kind == RawTokenKind::Gt {
                            i += 1;
                            span_index += 1;
                            TokenKind::RFArrow
                        } else {
                            TokenKind::Eq
                        },
                    RawTokenKind::Colon =>
                        if tokens[i].kind == RawTokenKind::Colon {
                            i += 1;
                            span_index += 1;
                            TokenKind::Dcolon
                        } else {
                            TokenKind::Colon
                        },
                    RawTokenKind::Minus =>
                        if tokens[i].kind == RawTokenKind::Gt {
                            i += 1;
                            span_index += 1;
                            TokenKind::RArrow
                        } else {
                            TokenKind::Minus
                        },

                    RawTokenKind::LineComment => continue,
                    RawTokenKind::BlockComment { terminated } => {
                        if !terminated {
                            panic!("unterminated block comment")
                        }
                        continue;
                    }
                    RawTokenKind::Ident =>
                        if let Some(&keyword) = KEYWORDS.get(slice) {
                            keyword
                        } else if slice == "_" {
                            TokenKind::Underscore
                        } else {
                            let symbol = with_interner(|interner| interner.intern(slice));
                            TokenKind::Ident(symbol)
                        },
                    RawTokenKind::RawIdent => todo!(),
                    RawTokenKind::Literal { kind, suffix_start } =>
                        TokenKind::Literal { kind, suffix_start },
                    RawTokenKind::Lifetime { .. } =>
                        todo!("maybe use lifetime syntax as generic parameter (like ocaml)"),
                    RawTokenKind::Semi => TokenKind::Semi,
                    RawTokenKind::Underscore => TokenKind::Underscore,
                    RawTokenKind::Comma => TokenKind::Comma,
                    RawTokenKind::Dot => TokenKind::Dot,
                    RawTokenKind::OpenParen => TokenKind::OpenParen,
                    RawTokenKind::CloseParen => TokenKind::CloseParen,
                    RawTokenKind::OpenBrace => TokenKind::OpenBrace,
                    RawTokenKind::CloseBrace => TokenKind::CloseBrace,
                    RawTokenKind::OpenBracket => TokenKind::OpenBracket,
                    RawTokenKind::CloseBracket => TokenKind::CloseBracket,
                    RawTokenKind::At => TokenKind::At,
                    RawTokenKind::Pound => TokenKind::Pound,
                    RawTokenKind::Tilde => TokenKind::Tilde,
                    RawTokenKind::Question => TokenKind::Question,
                    RawTokenKind::Dollar => TokenKind::Dollar,
                    RawTokenKind::Not => TokenKind::Not,
                    RawTokenKind::Lt => TokenKind::Lt,
                    RawTokenKind::Gt => TokenKind::Gt,
                    RawTokenKind::And => TokenKind::And,
                    RawTokenKind::Or => TokenKind::Or,
                    RawTokenKind::Plus => TokenKind::Plus,
                    RawTokenKind::Star => TokenKind::Star,
                    RawTokenKind::Slash => TokenKind::Slash,
                    RawTokenKind::Caret => TokenKind::Caret,
                    RawTokenKind::Percent => TokenKind::Percent,
                    RawTokenKind::Unknown => TokenKind::Unknown,
                };

                Token { span, kind }
            };
            vec.push(token)
        }

        // just manually add a <eof> token for easier parsing
        vec.push(Token { span: Span::new(file, span_index, span_index), kind: TokenKind::Eof });
        vec.into_iter()
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Ident(sym) => write!(f, "identifier `{}`", sym),
            _ => write!(f, "{:?}", self),
        }
    }
}
/// token kind that has been further processed to include keywords
#[derive(Debug, Hash, Clone, Copy, Eq, PartialEq)]
pub enum TokenKind {
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

impl TokenKind {
    pub fn is_delimiter(&self) -> bool {
        use TokenKind::*;
        matches!(self, OpenBrace | CloseBrace | OpenParen | CloseParen | OpenBracket | CloseBracket)
    }
}

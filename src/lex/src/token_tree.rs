use crate::{Token, TokenKind};
use std::fmt::{self, Debug, Formatter};
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub struct TokenStream {
    token_trees: Rc<Vec<TokenTree>>,
}

impl Debug for TokenStream {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenStream").finish()
    }
}

impl TokenStream {
    pub fn from_tokens(tokens: impl IntoIterator<Item = Token>) -> Self {
        let mut tokens = tokens.into_iter();
        let token_trees = vec![];
        loop {
            let token = match tokens.next() {
                Some(token) => token,
                None => break,
            };
            let token_tree = match token.kind {
                TokenKind::OpenParen => {}
                TokenKind::OpenBracket => {}
                TokenKind::OpenBrace => {}
                _ => todo!(),
            };
        }
        Self { token_trees: Rc::new(token_trees) }
    }
}

impl Iterator for TokenStream {
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenTree {
    Token(Token),
    Group(TokenGroup),
}

impl TokenTree {
    /// Consumes enough of the tokens to form a complete token tree.
    /// Returns `None` if the iterator is empty
    pub fn from_tokens(tokens: &mut impl Iterator<Item = Token>) -> Option<Self> {
        let mut tokens = tokens.peekable();
        let token = match tokens.next() {
            Some(token) => token,
            None => return None,
        };

        macro_rules! parse_group {
            ($close_token_kind:path) => {{
                let mut grouped_tokens = vec![];
                loop {
                    let token = match tokens.peek() {
                        Some(&token) => token,
                        None => todo!("unmatched delimiter"),
                    };
                    match token.kind {
                        $close_token_kind => break,
                        kind if kind.is_delimiter() => todo!("mismatched delimiter"),
                        _ => grouped_tokens.push(token),
                    }
                }
                let stream = TokenStream::from_tokens(grouped_tokens);
                TokenGroup { stream, delimiter: Delimiter::from($close_token_kind) }
            }};
        }

        let token_tree = match token.kind {
            TokenKind::OpenParen => todo!(),
            TokenKind::OpenBracket => parse_group!(TokenKind::CloseBracket),
            TokenKind::OpenBrace => todo!(),
            TokenKind::CloseParen => todo!(),
            TokenKind::CloseBracket => todo!(),
            TokenKind::CloseBrace => todo!(),
            _ => todo!(),
        };
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenGroup {
    delimiter: Delimiter,
    stream: TokenStream,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delimiter {
    Bracket,
    Brace,
    Paren,
}

impl From<TokenKind> for Delimiter {
    fn from(kind: TokenKind) -> Self {
        use TokenKind::*;
        match kind {
            OpenParen | CloseParen => Self::Paren,
            OpenBrace | CloseBrace => Self::Brace,
            OpenBracket | CloseBracket => Self::Bracket,
            _ => panic!("invalid delimiter"),
        }
    }
}

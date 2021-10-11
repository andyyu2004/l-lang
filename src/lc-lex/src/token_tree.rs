use crate::{Token, TokenKind};
use lc_span::Span;
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Index;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TokenStream {
    size: usize,
    token_trees: Rc<Vec<TokenTree>>,
}

impl TokenStream {
    pub fn new(token_trees: Rc<Vec<TokenTree>>) -> Self {
        let size = token_trees.iter().map(|tt| tt.size()).sum();
        Self { size, token_trees }
    }

    pub fn span(&self) -> Span {
        assert!(!self.is_empty(), "todo?");
        let tts = &self.token_trees;
        tts[0].span().merge(tts[tts.len() - 1].span())
    }

    pub fn is_empty(&self) -> bool {
        self.token_trees.is_empty()
    }

    /// The number of token trees in the stream
    pub fn len(&self) -> usize {
        self.token_trees.len()
    }

    /// The number of tokens in the stream
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Index<usize> for TokenStream {
    type Output = TokenTree;

    fn index(&self, index: usize) -> &Self::Output {
        &self.token_trees[index]
    }
}

#[derive(Default, Debug)]
pub struct TokenStreamBuilder {
    token_trees: Vec<TokenTree>,
}

impl TokenStreamBuilder {
    pub fn push(&mut self, tt: TokenTree) {
        self.token_trees.push(tt)
    }

    pub fn push_token(&mut self, token: Token) {
        self.push(TokenTree::Token(token))
    }

    pub fn to_stream(self) -> TokenStream {
        TokenStream::new(Rc::new(self.token_trees))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenTree {
    Token(Token),
    Group(TokenGroup),
}

impl TokenTree {
    pub fn span(&self) -> Span {
        match self {
            TokenTree::Token(token) => token.span,
            TokenTree::Group(group) => group.span(),
        }
    }

    pub fn try_into_token(self) -> Result<Token, Self> {
        if let Self::Token(v) = self { Ok(v) } else { Err(self) }
    }

    pub fn try_into_group(self) -> Result<TokenGroup, Self> {
        if let Self::Group(v) = self { Ok(v) } else { Err(self) }
    }

    pub fn as_token(&self) -> Option<&Token> {
        if let Self::Token(v) = self { Some(v) } else { None }
    }

    pub fn as_group(&self) -> Option<&TokenGroup> {
        if let Self::Group(v) = self { Some(v) } else { None }
    }

    fn size(&self) -> usize {
        match self {
            TokenTree::Token(..) => 1,
            TokenTree::Group(group) => group.size(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenGroup {
    size: usize,
    delimiter: Delimiter,
    stream: TokenStream,
}

impl TokenGroup {
    pub fn new(delimiter: Delimiter, stream: TokenStream) -> Self {
        let size = 2 + stream.size();
        Self { delimiter, stream, size }
    }

    /// The number of tokens in this group (including the two delimiters)
    pub fn size(&self) -> usize {
        self.size
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Delimiter {
    pub span: Span,
    pub kind: DelimiterKind,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DelimiterKind {
    Bracket,
    Brace,
    Paren,
}

impl DelimiterKind {
    pub fn close_token_kind(self) -> TokenKind {
        match self {
            DelimiterKind::Bracket => TokenKind::CloseBracket,
            DelimiterKind::Brace => TokenKind::CloseBrace,
            DelimiterKind::Paren => TokenKind::CloseParen,
        }
    }
}

impl Display for DelimiterKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            DelimiterKind::Bracket => "[",
            DelimiterKind::Brace => "{",
            DelimiterKind::Paren => "(",
        };
        write!(f, "{}", s)
    }
}

impl From<Token> for Delimiter {
    fn from(Token { span, kind }: Token) -> Self {
        Delimiter { span, kind: kind.into() }
    }
}

impl From<TokenKind> for DelimiterKind {
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

impl TokenGroup {
    #[inline(always)]
    pub fn span(&self) -> Span {
        self.delimiter.span
    }
}

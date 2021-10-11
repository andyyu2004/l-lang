use std::iter::Peekable;

use super::*;
use lc_lex::{Delimiter, TokenIterator, TokenStream, TokenStreamBuilder};
use lc_session::Session;

pub struct TokenTreeParser<'a> {
    sess: &'a Session,
    tokens: Peekable<TokenIterator>,
}

impl<'a> TokenTreeParser<'a> {
    pub fn new(sess: &'a Session, tokens: TokenIterator) -> Self {
        Self { sess, tokens: tokens.peekable() }
    }
}

impl<'a> TokenTreeParser<'a> {
    pub fn parse_token_stream(&mut self) -> TokenStream {
        let stream = self.parse_token_stream_inner();
        assert!(self.tokens.peek().is_none());
        stream
    }

    pub fn parse_token_stream_inner(&mut self) -> TokenStream {
        use TokenKind::*;
        let mut builder = TokenStreamBuilder::default();
        loop {
            let token: Token = match self.tokens.peek() {
                Some(&token) => token,
                None => break,
            };
            match token.kind {
                OpenBrace | OpenBracket | OpenParen =>
                    builder.push(TokenTree::Group(self.parse_token_group())),
                CloseBrace | CloseBracket | CloseParen => break,
                _ => {
                    self.tokens.next();
                    builder.push_token(token);
                }
            }
        }
        builder.to_stream()
    }

    pub fn parse_token_tree(&mut self) -> ParseResult<'a, TokenTree> {
        use TokenKind::*;
        let token: Token = match self.tokens.peek() {
            Some(&token) => token,
            None => return Err(self.sess.build_error(Span::default(), ParseError::Eof)),
        };
        match token.kind {
            OpenBrace | OpenBracket | OpenParen => Ok(TokenTree::Group(self.parse_token_group())),
            CloseBrace | CloseBracket | CloseParen => Err(self
                .sess
                .build_error(token.span, ParseError::UnmatchedCloseTokenTreeDelimiter(token.kind))),
            _ => {
                self.tokens.next();
                Ok(TokenTree::Token(token))
            }
        }
    }

    /// Expects the iterator to be non-empty and at the start of a group including the opening delimiter
    pub fn parse_token_group(&mut self) -> TokenGroup {
        let open_delimiter = self.tokens.next().unwrap();
        let stream = self.parse_token_stream_inner();
        // TODO should we consume the token if it is a mismatch or just peek it?
        let next = self.tokens.peek().copied();
        let open_delimiter: Delimiter = open_delimiter.into();
        match next {
            Some(token) => {
                let span = open_delimiter.span.merge(token.span);
                let expected_close_token_kind = open_delimiter.kind.close_token_kind();
                if token.kind != expected_close_token_kind {
                    let err = ParseError::MismatchedTokenTreeDelimiter(
                        expected_close_token_kind,
                        token.kind,
                    );
                    self.sess.emit_error(span, err);
                } else {
                    self.tokens.next();
                }

                let delimiter = Delimiter {
                    span: open_delimiter.span.merge(token.span),
                    kind: open_delimiter.kind,
                };
                TokenGroup::new(delimiter, stream)
            }
            None => {
                let span = open_delimiter.span.merge(stream.span());
                let err = ParseError::UnmatchedOpenTokenTreeDelimiter(open_delimiter.kind);
                self.sess.emit_error(span, err);
                let delimiter = Delimiter { span, kind: open_delimiter.kind };
                TokenGroup::new(delimiter, stream)
            }
        }
    }
}

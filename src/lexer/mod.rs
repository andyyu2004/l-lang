//! lexing (some from rustc)

mod cursor;
mod lexing;
mod span;
crate mod symbol;

crate use lexing::{LiteralKind, Token, TokenKind};
crate use span::Span;
crate use symbol::Symbol;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
crate struct Tok {
    pub span: Span,
    pub kind: TokenKind,
}

/// transforms the rustc token with len into one with span
crate fn lex(src: &str) -> impl Iterator<Item = Tok> + '_ {
    let mut i = 0;

    // note: it is important to filter after so that the spans are correct
    lexing::tokenize(src)
        .map(move |t| {
            let span = Span::new(i, i + t.len);
            i += t.len;
            Tok { span, kind: t.kind }
        })
        .filter(|t| t.kind != TokenKind::Whitespace)
}

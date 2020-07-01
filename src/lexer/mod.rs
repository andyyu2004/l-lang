//! lexing (some from rustc)

mod cursor;
mod lexer;
mod lexing;
crate mod symbol;

pub use lexing::{tokenize, Token, TokenKind};

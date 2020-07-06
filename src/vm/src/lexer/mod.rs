//! lexing (some from rustc)

mod cursor;
mod lexer;
mod lexing;
crate mod symbol;

crate use lexer::{Lexer, Tok, TokenType};
crate use lexing::LiteralKind;
use lexing::{Token, TokenKind};
crate use symbol::Symbol;

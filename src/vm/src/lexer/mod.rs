//! lexing (some from rustc)

mod cursor;
mod lexer;
mod lexing;
pub mod symbol;

pub use lexer::{Lexer, Tok, TokenType};
pub use lexing::{Base, LiteralKind};
use lexing::{Token, TokenKind};
pub use symbol::Symbol;

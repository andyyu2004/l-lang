mod interner;

use crate::span::with_interner;
pub use interner::Interner;
use std::fmt::{self, Display, Formatter};

// there is probably a better way than manually counting the symbol indices :)
// without proc macro?
pub const SYMBOLS: &[&str] = &["", "float", "bool", "char", "int", "main", "self", "Self"];
pub const EMPTY: Symbol = Symbol(0);
pub const FLOAT: Symbol = Symbol(1);
pub const BOOL: Symbol = Symbol(2);
pub const CHAR: Symbol = Symbol(3);
pub const INT: Symbol = Symbol(4);
pub const MAIN: Symbol = Symbol(5);
// upper and lower case self
pub const LSELF: Symbol = Symbol(6);
pub const USELF: Symbol = Symbol(7);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(pub usize);

impl Symbol {
    pub const fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn as_str(self) -> &'static str {
        with_interner(|interner| interner.get_str(self))
    }

    pub fn intern_str(string: &str) -> Self {
        with_interner(|interner| interner.intern(string))
    }

    pub fn intern(disp: impl Display) -> Self {
        Self::intern_str(&disp.to_string())
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

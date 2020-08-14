mod interner;

use crate::span::with_interner;
pub use interner::Interner;
use std::fmt::{self, Display, Formatter};

// there is probably a better way than manually counting the symbol indices :)
// without proc macro?
pub const SYMBOLS: &[&str] = &["number", "bool", "char", "main"];
pub const NUMBER: Symbol = Symbol(0);
pub const BOOL: Symbol = Symbol(1);
pub const CHAR: Symbol = Symbol(2);
pub const MAIN: Symbol = Symbol(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(pub usize);

impl Symbol {
    pub const fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn as_str(self) -> &'static str {
        with_interner(|interner| interner.get_str(self))
    }

    pub fn intern(string: &str) -> Self {
        with_interner(|interner| interner.intern(string))
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        println!("{}", self.as_str());
        write!(f, "{}", self.as_str())
    }
}

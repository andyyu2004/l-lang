mod interner;

crate use interner::Interner;
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
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "${}", self.0)
    }
}

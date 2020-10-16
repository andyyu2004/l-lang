use crate::SPAN_GLOBALS;
use arena::DroplessArena;
use rustc_hash::FxHashMap;
use std::fmt::{self, Debug, Display, Formatter};

pub const SYMBOLS: &[&str] =
    &["", "float", "bool", "char", "int", "main", "self", "Self", "_", "rc", "intrinsics", "print"];

pub mod sym {
    use super::Symbol;

    pub const EMPTY: Symbol = Symbol(0);
    pub const FLOAT: Symbol = Symbol(1);
    pub const BOOL: Symbol = Symbol(2);
    pub const CHAR: Symbol = Symbol(3);
    pub const INT: Symbol = Symbol(4);
    pub const MAIN: Symbol = Symbol(5);
    // upper and lower case self
    pub const LSELF: Symbol = Symbol(6);
    pub const USELF: Symbol = Symbol(7);
    pub const USCORE: Symbol = Symbol(8);
    pub const RC: Symbol = Symbol(9);
    pub const INTRINSICS: Symbol = Symbol(10);
    pub const PRINT: Symbol = Symbol(11);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(pub usize);

fn with_interner<T, F: FnOnce(&mut Interner) -> T>(f: F) -> T {
    SPAN_GLOBALS.with(|session_globals| f(&mut *session_globals.symbol_interner.borrow_mut()))
}

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
pub struct Interner {
    arena: DroplessArena,
    symbols: FxHashMap<&'static str, Symbol>,
    strs: Vec<&'static str>,
}

impl Debug for Interner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interner")
            .field("symbols", &self.symbols)
            .field("strs", &self.strs)
            .finish_non_exhaustive()
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::prefill(SYMBOLS)
    }
}

impl Interner {
    fn prefill(strs: &[&'static str]) -> Self {
        Self {
            arena: DroplessArena::default(),
            strs: strs.into(),
            symbols: strs.iter().copied().zip((0..).map(Symbol::new)).collect(),
        }
    }
}

impl Interner {
    pub fn intern(&mut self, string: &str) -> Symbol {
        if let Some(&symbol) = self.symbols.get(&string) {
            return symbol;
        }
        let symbol = Symbol::new(self.strs.len());
        let s: &str =
            unsafe { std::str::from_utf8_unchecked(self.arena.alloc_slice(string.as_bytes())) };
        // it is safe to cast to &'static as we will only access it while the arena is alive
        let ss: &'static str = unsafe { &*(s as *const str) };
        self.strs.push(ss);
        self.symbols.insert(ss, symbol);
        symbol
    }

    /// get the string which the symbol represents
    pub fn get_str(&self, symbol: Symbol) -> &'static str {
        self.strs[symbol.0]
    }
}

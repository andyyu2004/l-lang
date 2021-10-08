use crate::SPAN_GLOBALS;
use arena::DroplessArena;
use rustc_hash::FxHashMap;
use std::fmt::{self, Debug, Display, Formatter};

symbols! {
    Keywords {
        Empty: "",
        USelf: "Self",
        LSelf: "self",
    }
    // the following must be in alphabetical order
    Symbols {
        addr,
        bool,
        char,
        float,
        int,
        intrinsics,
        main,
        print,
        rc,
    }
}

pub mod sym {
    use super::*;
    define_symbols!();
}

pub mod kw {
    use super::*;
    keywords!();
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

    pub fn intern(string: &str) -> Self {
        with_interner(|interner| interner.intern(string))
    }

    pub fn intern_str(s: &str) -> &'static str {
        with_interner(|interner| interner.intern_str(s))
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
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
        // fresh is defined in the `symbol!` macro
        Self::fresh()
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

    pub fn intern_str(&mut self, s: &str) -> &'static str {
        if s.is_empty() {
            return "";
        }

        let interned =
            unsafe { std::str::from_utf8_unchecked(self.arena.alloc_slice(s.as_bytes())) };
        unsafe { &*(interned as *const str) }
    }

    /// get the string which the symbol represents
    pub fn get_str(&self, symbol: Symbol) -> &'static str {
        self.strs[symbol.0]
    }
}

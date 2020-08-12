use super::{Symbol, SYMBOLS};
use crate::arena::DroplessArena;
use rustc_hash::FxHashMap;
use std::fmt::{self, Debug, Formatter};

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
        if let Some(&name) = self.symbols.get(&string) {
            return name;
        }
        let symbol = Symbol::new(self.strs.len());
        let s: &str =
            unsafe { std::str::from_utf8_unchecked(self.arena.alloc_slice(string.as_bytes())) };
        // it is safe to cast to &'static as we will only access it while the arena is alive
        let ss: &'static str = unsafe { &*(string as *const str) };
        self.strs.push(ss);
        self.symbols.insert(ss, symbol);
        symbol
    }

    /// get the string which the symbol represents
    pub fn get_str(&self, symbol: Symbol) -> &'static str {
        self.strs[symbol.0]
    }
}

use super::Symbol;
use rustc_ap_arena::DroplessArena;
use rustc_hash::FxHashMap;

#[derive(Default)]
crate struct Interner {
    arena: DroplessArena,
    symbols: FxHashMap<&'static str, Symbol>,
    strs: Vec<&'static str>,
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
    pub fn get_str(&self, symbol: Symbol) -> &str {
        self.strs[symbol.0]
    }
}

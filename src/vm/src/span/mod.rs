mod ctx;
mod source_map;

use crate::lexer::symbol;
pub use ctx::Ctx;
pub use source_map::SourceMap;
use std::cell::RefCell;

#[derive(Default, Debug)]
pub struct Globals {
    pub symbol_interner: RefCell<symbol::Interner>,
}

pub fn with_interner<R>(f: impl FnOnce(&mut symbol::Interner) -> R) -> R {
    GLOBALS.with(|globals| f(&mut globals.symbol_interner.borrow_mut()))
}

thread_local!(pub static GLOBALS: Globals = Default::default());

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Span {
    /// lo is inclusive
    pub lo: usize,
    /// lo is exclusive
    pub hi: usize,
}

impl Span {
    pub const fn new(lo: usize, hi: usize) -> Self {
        assert!(lo <= hi);
        Self { lo, hi }
    }

    pub fn merge(self, with: Span) -> Self {
        Self { lo: self.lo, hi: with.hi }
    }
}

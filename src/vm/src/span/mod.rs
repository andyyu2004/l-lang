mod ctx;
mod source_map;

use crate::lexer::symbol::{self, Symbol};
use crate::span;
pub use ctx::Ctx;
pub use source_map::SourceMap;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Default, Debug)]
pub struct Globals {
    pub symbol_interner: RefCell<symbol::Interner>,
    pub source_map: RefCell<Option<Rc<SourceMap>>>,
}

pub fn with_interner<R>(f: impl FnOnce(&mut symbol::Interner) -> R) -> R {
    GLOBALS.with(|globals| f(&mut globals.symbol_interner.borrow_mut()))
}

pub fn with_source_map<R>(f: impl FnOnce(&SourceMap) -> R) -> R {
    GLOBALS.with(|globals| f(&mut globals.source_map.borrow().as_ref().unwrap()))
}

thread_local!(pub static GLOBALS: Globals = Default::default());

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Span {
    /// lo is inclusive
    pub lo: usize,
    /// lo is exclusive
    pub hi: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", with_source_map(|map| map.span_to_string(*self)))
    }
}

impl Span {
    pub const fn new(lo: usize, hi: usize) -> Self {
        assert!(lo <= hi);
        Self { lo, hi }
    }

    pub const fn empty() -> Self {
        Self::new(0, 0)
    }

    pub fn hi(self) -> Self {
        Self::new(self.hi, self.hi)
    }

    /// assumes `self.lo <= with.lo && with.hi >= self.hi`
    pub fn merge(self, with: Span) -> Self {
        Self { lo: self.lo, hi: with.hi }
    }

    pub fn intern(self) -> Symbol {
        with_source_map(|map| with_interner(|interner| interner.intern(map.span_to_slice(self))))
    }
}

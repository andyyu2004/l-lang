mod ctx;
mod source_map;

use crate::lexer::symbol;
use crate::span;
pub use ctx::Ctx;
pub use source_map::SourceMap;
use std::cell::RefCell;
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

impl Span {
    pub const fn new(lo: usize, hi: usize) -> Self {
        assert!(lo <= hi);
        Self { lo, hi }
    }

    pub fn hi(self) -> Self {
        Self::new(self.hi, self.hi)
    }

    pub fn merge(self, with: Span) -> Self {
        Self { lo: self.lo, hi: with.hi }
    }

    pub fn to_string(self) -> String {
        with_source_map(|map| map.span_to_string(self))
    }
}

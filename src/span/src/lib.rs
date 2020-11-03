#![feature(debug_non_exhaustive)]
#![feature(const_panic)]

#[macro_use]
extern crate macros;

mod source_map;
mod symbol;

pub use source_map::SourceMap;
pub use symbol::{kw, sym, Symbol};

use codespan::ByteIndex;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[derive(Default, Debug)]
pub struct SpanGlobals {
    pub symbol_interner: RefCell<symbol::Interner>,
    pub source_map: RefCell<Option<Rc<SourceMap>>>,
}

pub fn with_interner<R>(f: impl FnOnce(&mut symbol::Interner) -> R) -> R {
    SPAN_GLOBALS.with(|globals| f(&mut globals.symbol_interner.borrow_mut()))
}

pub fn with_source_map<R>(f: impl FnOnce(&SourceMap) -> R) -> R {
    SPAN_GLOBALS.with(|globals| f(&mut globals.source_map.borrow().as_ref().unwrap()))
}

thread_local!(pub static SPAN_GLOBALS: SpanGlobals = Default::default());

/// thin wrapper around codespan::Span for convenience
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default)]
pub struct Span(codespan::Span);

pub trait SpanIdx {
    fn into(self) -> ByteIndex;
}

impl SpanIdx for ByteIndex {
    fn into(self) -> ByteIndex {
        self
    }
}

impl SpanIdx for usize {
    fn into(self) -> ByteIndex {
        (self as u32).into()
    }
}

impl Span {
    pub fn new(start: impl SpanIdx, end: impl SpanIdx) -> Self {
        Self(codespan::Span::new(start.into(), end.into()))
    }

    pub fn intern(self) -> Symbol {
        with_source_map(|map| with_interner(|interner| interner.intern(map.span_to_slice(self))))
    }

    pub fn is_empty(&self) -> bool {
        self.start() == self.end()
    }

    pub fn merge(self, other: Self) -> Self {
        Self(self.0.merge(other.0))
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", with_source_map(|smap| smap.span_to_string(*self)))
    }
}

impl Deref for Span {
    type Target = codespan::Span;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Span {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

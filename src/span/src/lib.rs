#![feature(debug_non_exhaustive)]
#![feature(min_specialization)]
#![feature(type_ascription)]
#![feature(const_panic)]

#[macro_use]
extern crate macros;

mod source_map;
mod symbol;

use codespan_reporting::diagnostic::Label;
pub use source_map::{FileIdx, SourceMap, ROOT_FILE_IDX};
pub use symbol::{kw, sym, Symbol};

use codespan::ByteIndex;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Default, Debug)]
pub struct SpanGlobals {
    pub symbol_interner: RefCell<symbol::Interner>,
    pub source_map: RefCell<SourceMap>,
}

pub fn with_interner<R>(f: impl FnOnce(&mut symbol::Interner) -> R) -> R {
    SPAN_GLOBALS.with(|globals| f(&mut globals.symbol_interner.borrow_mut()))
}

pub fn with_source_map<R>(f: impl FnOnce(&mut SourceMap) -> R) -> R {
    SPAN_GLOBALS.with(|globals| f(&mut *globals.source_map.borrow_mut()))
}

thread_local!(pub static SPAN_GLOBALS: SpanGlobals = Default::default());

impl Into<Label<FileIdx>> for Span {
    fn into(self) -> Label<FileIdx> {
        Label::primary(self.file, *self)
    }
}

/// thin wrapper around codespan::Span for convenience
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Span {
    pub file: FileIdx,
    span: codespan::Span,
}

impl Default for Span {
    fn default() -> Self {
        Self { file: ROOT_FILE_IDX, span: Default::default() }
    }
}

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
    pub fn new(file: FileIdx, start: impl SpanIdx, end: impl SpanIdx) -> Self {
        Self { file, span: codespan::Span::new(start.into(), end.into()) }
    }

    pub fn intern(self) -> Symbol {
        with_source_map(|map| with_interner(|interner| interner.intern(map.span_to_slice(self))))
    }

    pub fn is_empty(&self) -> bool {
        self.start() == self.end()
    }

    pub fn merge(self, other: Self) -> Self {
        assert_eq!(self.file, other.file);
        Self { file: self.file, span: self.span.merge(other.span) }
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
        &self.span
    }
}

impl DerefMut for Span {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.span
    }
}

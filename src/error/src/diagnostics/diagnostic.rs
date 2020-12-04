use crate::{Emitter, LError, LResult, TextEmitter};
use span::Span;
use std::error::Error;
use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;
use std::{
    cell::{Cell, RefCell}, ops::DerefMut
};

#[derive(Default)]
pub struct Diagnostics {
    err_count: Cell<usize>,
}

impl Diagnostics {
    crate fn inc_err_count(&self) {
        self.err_count.set(1 + self.err_count.get());
    }

    pub fn try_run<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        let old = self.err_count();
        let ret = f();
        // no errors have occured during `f`
        if self.err_count() == old { Some(ret) } else { None }
    }

    pub fn err_count(&self) -> usize {
        self.err_count.get()
    }

    pub fn has_errors(&self) -> bool {
        self.err_count.get() > 0
    }

    pub fn check_for_errors(&self) -> LResult<()> {
        if self.has_errors() { Err(LError::ErrorReported) } else { Ok(()) }
    }

    pub fn build_error(
        &self,
        span: impl Into<MultiSpan>,
        err: impl Error,
    ) -> DiagnosticBuilder<'_> {
        DiagnosticBuilder::new(self, span, err)
    }

    pub fn emit_error(&self, span: impl Into<MultiSpan>, err: impl Error) -> LError {
        self.build_error(span, err).emit();
        LError::ErrorReported
    }
}

/// a single diagnostic error message
#[derive(Debug)]
pub struct Diagnostic {
    /// the main error of the diagnostic
    crate error: String,
    crate spans: Vec<Span>,
    crate labelled_spans: Vec<(Span, String)>,
    crate notes: Vec<String>,
}

#[derive(Debug)]
pub struct MultiSpan {
    spans: Vec<Span>,
}

impl From<Vec<Span>> for MultiSpan {
    fn from(spans: Vec<Span>) -> Self {
        Self { spans }
    }
}

impl From<Span> for MultiSpan {
    fn from(span: Span) -> Self {
        Self { spans: vec![span] }
    }
}

impl Diagnostic {
    pub fn from_err(spans: impl Into<MultiSpan>, error: impl Error) -> Self {
        let multispan = spans.into();
        Self {
            labelled_spans: Default::default(),
            notes: Default::default(),
            error: error.to_string(),
            spans: multispan.spans,
        }
    }

    pub fn get_first_span(&self) -> Span {
        self.spans[0]
    }
}

pub struct DiagnosticBuilder<'a> {
    diagnostics: &'a Diagnostics,
    diagnostic: Diagnostic,
    emitter: RefCell<Box<dyn Emitter>>,
}

// builder methods
impl<'a> DiagnosticBuilder<'a> {
    pub fn labelled_span(&mut self, span: Span, msg: String) -> &mut Self {
        self.labelled_spans.push((span, msg));
        self
    }

    pub fn span(&mut self, span: Span) -> &mut Self {
        self.spans.push(span);
        self
    }

    pub fn note(&mut self, note: &str) -> &mut Self {
        self.notes.push(note.to_owned());
        self
    }
}
impl<'a> Debug for DiagnosticBuilder<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.diagnostic)
    }
}

impl<'a> Deref for DiagnosticBuilder<'a> {
    type Target = Diagnostic;

    fn deref(&self) -> &Self::Target {
        &self.diagnostic
    }
}

impl<'a> DerefMut for DiagnosticBuilder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.diagnostic
    }
}

impl<'a> DiagnosticBuilder<'a> {
    fn default_emitter() -> RefCell<Box<dyn Emitter>> {
        RefCell::new(box TextEmitter::default())
    }

    pub fn emit(&self) {
        self.diagnostics.inc_err_count();
        self.emitter.borrow_mut().emit(self)
    }

    crate fn new(
        diagnostics: &'a Diagnostics,
        span: impl Into<MultiSpan>,
        err: impl Error,
    ) -> Self {
        let diagnostic = Diagnostic::from_err(span, err);
        Self { diagnostics, diagnostic, emitter: Self::default_emitter() }
    }
}

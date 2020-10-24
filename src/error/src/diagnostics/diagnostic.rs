use crate::{Emitter, LError, LResult, TextEmitter};
use span::Span;
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;

#[derive(Default)]
pub struct Diagnostics {
    err_count: Cell<usize>,
}

impl Diagnostics {
    pub(super) fn inc_err_count(&self) {
        self.err_count.set(1 + self.err_count.get());
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
#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct Diagnostic {
    pub messages: Vec<String>,
    pub span: MultiSpan,
}

impl Diagnostic {
    pub fn from_err(span: impl Into<MultiSpan>, err: impl Error) -> Self {
        let span = span.into();
        Self { messages: vec![format!("{}", err)], span }
    }

    pub fn get_span(&self) -> Span {
        self.span.primary_spans[0]
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct MultiSpan {
    pub primary_spans: Vec<Span>,
}

impl<I> From<I> for MultiSpan
where
    I: IntoIterator<Item = Span>,
{
    fn from(iter: I) -> Self {
        Self { primary_spans: iter.into_iter().collect() }
    }
}

pub struct DiagnosticBuilder<'a> {
    diagnostics: &'a Diagnostics,
    diagnostic: Diagnostic,
    emitter: RefCell<Box<dyn Emitter>>,
}

impl<'a> Debug for DiagnosticBuilder<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.diagnostic)
    }
}

impl Deref for DiagnosticBuilder<'_> {
    type Target = Diagnostic;

    fn deref(&self) -> &Self::Target {
        &self.diagnostic
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

    pub(super) fn new(
        diagnostics: &'a Diagnostics,
        span: impl Into<MultiSpan>,
        err: impl Error,
    ) -> Self {
        let diagnostic = Diagnostic::from_err(span, err);
        Self { diagnostics, diagnostic, emitter: Self::default_emitter() }
    }
}

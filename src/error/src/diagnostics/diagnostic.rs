use crate::{Emitter, LError, LResult, TextEmitter};
use codespan_reporting::diagnostic::{Label, LabelStyle};
use span::{FileIdx, Span};
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;

#[derive(Default)]
pub struct Diagnostics {
    err_count: Cell<usize>,
}

impl Diagnostics {
    crate fn inc_err_count(&self) {
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

type DiagnosticInner = codespan_reporting::diagnostic::Diagnostic<FileIdx>;

/// a single diagnostic error message
#[derive(Clone, Debug)]
pub struct Diagnostic {
    crate inner: DiagnosticInner,
}

impl Diagnostic {
    pub fn from_err(span: impl Into<MultiSpan>, err: impl Error) -> Self {
        let span = span.into();
        let inner = DiagnosticInner::error().with_message(err.to_string()).with_labels(
            span.primary_spans
                .iter()
                .map(|&span| Label::new(LabelStyle::Primary, span.file, *span))
                .collect(),
        );
        Self { inner }
    }

    pub fn get_first_span(&self) -> Span {
        let label = self.inner.labels.iter().nth(0).unwrap();
        Span::new(label.file_id, label.range.start, label.range.end)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct MultiSpan {
    pub primary_spans: Vec<Span>,
}

// only impl for vectors to avoid overlapping impls
impl From<Vec<Span>> for MultiSpan {
    default fn from(primary_spans: Vec<Span>) -> Self {
        Self { primary_spans }
    }
}

impl From<Span> for MultiSpan {
    fn from(span: Span) -> Self {
        Self { primary_spans: vec![span] }
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

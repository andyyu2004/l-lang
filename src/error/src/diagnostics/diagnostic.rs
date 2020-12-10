use crate::{Emitter, ErrorFormat, ErrorReported, JsonEmitter, LResult, TextEmitter};
use codespan_reporting::diagnostic::Severity;
use span::Span;
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};

pub struct Diagnostics {
    emitter: RefCell<Box<dyn Emitter>>,
    // maybe not worth generalizing to a hashmap as we only care about these two counts
    // and not the other levels of severity
    error_count: Cell<usize>,
    warning_count: Cell<usize>,
}

impl Diagnostics {
    pub fn with_error_format(error_format: ErrorFormat) -> Self {
        let emitter: Box<dyn Emitter> = match error_format {
            ErrorFormat::Text => box TextEmitter::default(),
            ErrorFormat::Json => box JsonEmitter::default(),
        };

        Self {
            emitter: RefCell::new(emitter),
            error_count: Default::default(),
            warning_count: Default::default(),
        }
    }

    crate fn inc_err_count(&self) {
        self.error_count.set(1 + self.error_count.get());
    }

    crate fn inc_warning_count(&self) {
        self.warning_count.set(1 + self.warning_count.get());
    }

    pub fn try_run<R>(&self, f: impl FnOnce() -> R) -> Result<R, R> {
        let old = self.err_count();
        let ret = f();
        // no errors have occured during `f`
        if self.err_count() == old { Ok(ret) } else { Err(ret) }
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count.get()
    }

    pub fn err_count(&self) -> usize {
        self.error_count.get()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count.get() > 0
    }

    pub fn check_for_errors(&self) -> LResult<()> {
        if self.has_errors() { Err(ErrorReported) } else { Ok(()) }
    }

    pub fn build_error(
        &self,
        span: impl Into<MultiSpan>,
        err: impl Error,
    ) -> DiagnosticBuilder<'_> {
        DiagnosticBuilder::new(self, Severity::Error, span, err)
    }

    pub fn emit_warning(&self, span: impl Into<MultiSpan>, err: impl Error) {
        DiagnosticBuilder::new(self, Severity::Warning, span, err).emit()
    }

    pub fn emit_error(&self, span: impl Into<MultiSpan>, err: impl Error) -> ErrorReported {
        self.build_error(span, err).emit();
        ErrorReported
    }
}

/// a single diagnostic error message
#[derive(Debug)]
pub struct Diagnostic {
    /// the primary message of the diagnostic
    crate severity: Severity,
    crate msg: String,
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
    pub fn from_err(severity: Severity, spans: impl Into<MultiSpan>, error: impl Error) -> Self {
        let multispan = spans.into();
        Self {
            severity,
            msg: error.to_string(),
            spans: multispan.spans,
            labelled_spans: Default::default(),
            notes: Default::default(),
        }
    }

    pub fn get_first_span(&self) -> Span {
        self.spans[0]
    }
}

pub struct DiagnosticBuilder<'a> {
    diagnostics: &'a Diagnostics,
    diagnostic: Diagnostic,
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
    pub fn emit(&self) {
        match self.diagnostic.severity {
            Severity::Error => self.diagnostics.inc_err_count(),
            Severity::Warning => self.diagnostics.inc_warning_count(),
            _ => {}
        }
        self.diagnostics.emitter.borrow_mut().emit(self)
    }

    crate fn new(
        diagnostics: &'a Diagnostics,
        severity: Severity,
        span: impl Into<MultiSpan>,
        err: impl Error,
    ) -> Self {
        let diagnostic = Diagnostic::from_err(severity, span, err);
        Self { diagnostics, diagnostic }
    }
}

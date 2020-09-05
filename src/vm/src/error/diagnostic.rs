use super::DiagnosticBuilder;
use crate::error::{LError, LResult};
use crate::span::Span;
use std::cell::Cell;
use std::error::Error;

#[derive(Default)]
pub struct Diagnostics {
    err_count: Cell<usize>,
}

impl Diagnostics {
    fn inc_err_count(&self) {
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

    pub fn build_error(&self, span: Span, err: impl Error) -> DiagnosticBuilder {
        self.inc_err_count();
        DiagnosticBuilder::from_err(span, err)
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
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct MultiSpan {
    pub primary_spans: Vec<Span>,
}

impl From<Span> for MultiSpan {
    fn from(span: Span) -> Self {
        Self { primary_spans: vec![span] }
    }
}

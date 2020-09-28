use super::{diagnostic::MultiSpan, Diagnostic, Diagnostics, Emitter, TextEmitter};
use crate::span::Span;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;

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

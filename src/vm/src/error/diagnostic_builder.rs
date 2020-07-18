use super::{Diagnostic, Emitter, TextEmitter};
use crate::span::Span;
use std::cell::RefCell;
use std::error::Error;
use std::ops::Deref;

pub struct DiagnosticBuilder {
    diagnostic: Diagnostic,
    emitter: RefCell<Box<dyn Emitter>>,
}

impl Default for DiagnosticBuilder {
    fn default() -> Self {
        Self { diagnostic: Diagnostic::default(), emitter: Self::default_emitter() }
    }
}

impl Deref for DiagnosticBuilder {
    type Target = Diagnostic;

    fn deref(&self) -> &Self::Target {
        &self.diagnostic
    }
}

impl DiagnosticBuilder {
    fn default_emitter() -> RefCell<Box<dyn Emitter>> {
        RefCell::new(box TextEmitter::default())
    }

    pub fn emit(&self) {
        self.emitter.borrow_mut().emit(self)
    }

    pub fn new_diagnostic(diagnostic: Diagnostic) -> Self {
        Self { diagnostic, emitter: Self::default_emitter() }
    }

    pub fn from_err(span: Span, err: impl Error) -> Self {
        let diagnostic = Diagnostic::from_err(span, err);
        Self::new_diagnostic(diagnostic)
    }
}

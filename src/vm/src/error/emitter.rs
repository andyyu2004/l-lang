use super::Diagnostic;
use crate::span::Span;

/// trait for an object that formats diagnostics
pub trait Emitter {
    fn emit(&mut self, diagnostics: &Diagnostic);
}

/// emitter for text/tty based interface
#[derive(Default)]
pub struct TextEmitter {}

impl TextEmitter {
    fn emit_span(&mut self, span: Span) {
        println!("{}", span.to_string())
    }
}

impl Emitter for TextEmitter {
    fn emit(&mut self, diagnostic: &Diagnostic) {
        red!("error: ");
        let Diagnostic { span, messages } = diagnostic;
        for message in messages {
            red_ln!("{}", message)
        }
        span.primary_spans.iter().for_each(|&s| self.emit_span(s));
        println!()
    }
}

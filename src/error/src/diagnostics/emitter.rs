use crate::Diagnostic;
use span::Span;

/// trait for an object that formats diagnostics
pub trait Emitter {
    fn emit(&mut self, diagnostics: &Diagnostic);
}

/// emitter for text/tty based interface
#[derive(Default)]
pub struct TextEmitter {}

impl TextEmitter {
    fn emit_span(&mut self, span: Span) {
        eprintln!("source: {}", span.to_string())
    }
}

impl Emitter for TextEmitter {
    fn emit(&mut self, diagnostic: &Diagnostic) {
        e_red!("error: {:?}", diagnostic);
        // let Diagnostic { span, messages } = diagnostic;
        // for message in messages {
        //     e_red_ln!("{}", message)
        // }
        // span.primary_spans.iter().for_each(|&s| self.emit_span(s));
        // eprintln!()
    }
}

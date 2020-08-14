use super::Diagnostic;

/// trait for an object that formats diagnostics
pub trait Emitter {
    fn emit(&mut self, diagnostics: &Diagnostic);
}

/// emitter for text/tty based interface
#[derive(Default)]
pub struct TextEmitter {}

impl Emitter for TextEmitter {
    fn emit(&mut self, diagnostic: &Diagnostic) {
        println!("{:?}", diagnostic.messages)
    }
}

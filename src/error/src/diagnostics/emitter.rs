use crate::Diagnostic;
use codespan_reporting::term::{emit, Config};
use termcolor::{BufferedStandardStream, ColorChoice};

/// trait for an object that formats diagnostics
pub trait Emitter {
    fn emit(&mut self, diagnostics: &Diagnostic);
}

/// emitter for text/tty based interface
#[derive(Default)]
pub struct TextEmitter {}

impl Emitter for TextEmitter {
    fn emit(&mut self, diagnostic: &Diagnostic) {
        let mut writer = BufferedStandardStream::stderr(ColorChoice::Auto);
        span::with_source_map(|files| {
            emit(&mut writer, &Config::default(), files, &diagnostic.inner)
        })
        .unwrap()
    }
}

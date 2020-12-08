use crate::Diagnostic;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::term::{emit, Config};
use span::FileIdx;
use termcolor::{BufferedStandardStream, ColorChoice};

/// trait for an object that formats diagnostics
pub trait Emitter {
    fn emit(&mut self, diagnostics: &Diagnostic);
}

/// emitter for text/tty based interface
#[derive(Default)]
pub struct TextEmitter {}

type DiagnosticInner = codespan_reporting::diagnostic::Diagnostic<FileIdx>;

impl Emitter for TextEmitter {
    fn emit(&mut self, diagnostic: &Diagnostic) {
        let mut writer = BufferedStandardStream::stderr(ColorChoice::Auto);

        // convert our representation of a diagnostic into the codespan one
        // the labels consist of the labelled spans as well as some spans with no label
        let labels = diagnostic
            .labelled_spans
            .iter()
            .map(|(span, msg)| Label::primary(span.file, **span).with_message(msg))
            .chain(diagnostic.spans.iter().map(|&span| span.into()))
            .collect();

        let diag = DiagnosticInner::new(diagnostic.severity)
            .with_message(&diagnostic.msg)
            .with_labels(labels)
            .with_notes(diagnostic.notes.clone());

        span::with_source_map(|files| emit(&mut writer, &Config::default(), files, &diag)).unwrap()
    }
}

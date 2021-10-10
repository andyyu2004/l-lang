use crate::Diagnostic;
use codespan_reporting::diagnostic::{Label, Severity};
use codespan_reporting::files::Files;
use codespan_reporting::term::{emit, Config};
use lc_span::FileIdx;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use termcolor::{BufferedStandardStream, ColorChoice};

/// trait for an object that formats diagnostics
pub trait Emitter {
    fn emit(&mut self, diagnostics: &Diagnostic);
}

type DiagnosticInner = codespan_reporting::diagnostic::Diagnostic<FileIdx>;

/// emitter for text/tty based interface
#[derive(Default)]
pub struct TextEmitter {}

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

        lc_span::with_source_map(|files| emit(&mut writer, &Config::default(), files, &diag))
            .unwrap()
    }
}

/// json representation of a diagnostic
/// currently only contains the primary message
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonDiagnostic {
    pub severity: Severity,
    pub file: PathBuf,
    pub msg: String,
    pub line: usize,
}

/// emitter for text/tty based interface
#[derive(Default)]
pub struct JsonEmitter {
    // we accumulate the errors and emit them all at once at the end
    // so we can format them properly as a json array
    errors: Vec<JsonDiagnostic>,
}

impl Emitter for JsonEmitter {
    fn emit(&mut self, diagnostic: &Diagnostic) {
        lc_span::with_source_map(|files| {
            let span = diagnostic.get_first_span();
            let file = files.path_of(span.file).to_owned();
            let line = 1 + files.line_index(span.file, span.start().to_usize()).unwrap();
            self.errors.push(JsonDiagnostic {
                file,
                line,
                severity: diagnostic.severity,
                msg: diagnostic.msg.to_owned(),
            });
        });
    }
}

impl Drop for JsonEmitter {
    fn drop(&mut self) {
        eprintln!("{}", serde_json::to_string_pretty(&self.errors).unwrap());
        drop(&mut self.errors)
    }
}

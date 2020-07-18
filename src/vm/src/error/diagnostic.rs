use crate::span::Span;
use std::error::Error;

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
    primary_spans: Vec<Span>,
}

impl From<Span> for MultiSpan {
    fn from(span: Span) -> Self {
        Self { primary_spans: vec![span] }
    }
}

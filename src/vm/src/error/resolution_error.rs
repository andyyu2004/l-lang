use crate::span::Span;

crate type ResolutionResult<T> = Result<T, ResolutionError>;

#[derive(Debug)]
crate struct ResolutionError {
    span: Span,
    kind: ResolutionErrorKind,
}

impl ResolutionError {
    pub fn unbound_variable(span: Span) -> Self {
        Self { span, kind: ResolutionErrorKind::NotFound }
    }
}

#[derive(Debug)]
crate enum ResolutionErrorKind {
    NotFound,
}

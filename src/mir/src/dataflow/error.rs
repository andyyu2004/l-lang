use span::Span;
use thiserror::Error;

#[derive(Error, Debug)]
crate enum MirError {
    #[error("use of uninitialized variable `{}`", .0.to_string())]
    UninitializedVariable(Span),
}

use thiserror::Error;

#[derive(Error, Debug)]
crate enum MirError {
    #[error("use of uninitiazed variable")]
    UninitializedVariable,
}

use crate::impl_from_inner;

crate type TypeResult<T> = Result<T, TypeError>;
crate type InferResult<T> = Result<T, InferError>;

impl_from_inner!(InferError, TypeError, InferError);

#[derive(Debug)]
crate enum TypeError {
    InferError(InferError),
}

#[derive(Debug)]
crate enum InferError {
    UnificationFailure(),
}

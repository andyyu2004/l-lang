use crate::ty::{Ty, TyVid};
use ast::Ident;
use error::LError;
use ir::{self, Res};
use thiserror::Error;

pub type TypeResult<'tcx, T> = Result<T, TypeError<'tcx>>;

#[derive(Debug, Error)]
pub enum TypeError<'tcx> {
    #[error("failed to unify type `{0}` with `{1}`")]
    UnificationFailure(Ty<'tcx>, Ty<'tcx>),
    #[error("expected type `{0}`, found `{1}`")]
    Mismatch(Ty<'tcx>, Ty<'tcx>),
    #[error("expected `{0}-tuple`, found `{1}-tuple`")]
    TupleSizeMismatch(usize, usize),
    #[error("unknown field `{1}` on `{0}`")]
    UnknownField(Ty<'tcx>, Ident),
    #[error("cannot access fields on `{0}`")]
    BadFieldAccess(Ty<'tcx>),
    #[error("attempted to index `.{0}` into a {1}-tuple")]
    TupleOutOfBounds(usize, usize),
    #[error("expected {0} generic parameter{} but received {1}", util::pluralize!({*.0}))]
    GenericArgCount(usize, usize),
    #[error("{0}")]
    Msg(String),
    #[error("cannot dereference type `{0}`")]
    InvalidDereference(Ty<'tcx>),
    #[error("field `{0}` already declared in `{1}`")]
    FieldAlreadyDeclared(Ident, Ident),
    #[error("expected unit variant, found {0}")]
    UnexpectedVariant(Res),
    #[error("occurs check failed: type variable `{0}` occurs in type `{1}`")]
    OccursCheck(TyVid, Ty<'tcx>),
    #[error("operation requires unsafe context")]
    RequireUnsafeCtx,
    #[error("type annotations required")]
    InferenceFailure,
}

impl<'tcx> LError for TypeError<'tcx> {
    fn title(&self) -> &str {
        match self {
            TypeError::TupleSizeMismatch(..) | TypeError::Mismatch(..) => "type mismatch",
            TypeError::InferenceFailure => "inference failure",
            TypeError::RequireUnsafeCtx => "",
            _ => "type error",
        }
    }
}

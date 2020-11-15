use crate::ty::TyCtx;
use serde::Serializer;

pub trait TyEncoder<'tcx>: Serializer {
    fn tcx() -> TyCtx<'tcx>;
}

use crate::ty::*;
use serde::{Deserializer, Serializer};

pub trait TyEncoder<'tcx>: Serializer {
    fn tcx() -> TyCtx<'tcx>;
}

pub trait TyDecoder<'tcx>: Deserializer<'tcx> {
    fn tcx(&self) -> TyCtx<'tcx>;
}

pub trait TyDecodable<'tcx, D: TyDecoder<'tcx>>: Sized {
    fn decode(d: &mut D) -> Result<Self, D::Error>;
}

impl<'tcx, D: TyDecoder<'tcx>> TyDecodable<'tcx, D> for TyKind<'tcx> {
    fn decode(_d: &mut D) -> Result<Self, D::Error> {
        todo!()
    }
}

impl<'tcx, D: TyDecoder<'tcx>> TyDecodable<'tcx, D> for SubstsRef<'tcx> {
    fn decode(d: &mut D) -> Result<Self, D::Error> {
        d.tcx().mk_substs(vec![]);
        todo!()
    }
}

impl<'tcx, D: TyDecoder<'tcx>> TyDecodable<'tcx, D> for Ty<'tcx> {
    fn decode(_d: &mut D) -> Result<Self, D::Error> {
        todo!()
    }
}

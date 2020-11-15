use ir::DefId;
use lcore::ty::Ty;
use rustc_hash::FxHashMap;

#[macro_use]
extern crate serde;

/// a representation of everything you would need to know about a given package
#[derive(Debug, Serialize)]
pub struct PkgMetadata {
    pkg: Pkg<'static>,
}

#[derive(Debug, Serialize)]
pub struct Pkg<'tcx> {
    types: FxHashMap<DefId, Ty<'tcx>>,
}

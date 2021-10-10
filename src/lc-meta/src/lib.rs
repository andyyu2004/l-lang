#[macro_use]
extern crate serde;
extern crate lc_ir as ir;

use ir::DefId;
use lc_core::ty::Ty;
use rustc_hash::FxHashMap;

/// a representation of everything you would need to know about a given package
#[derive(Debug, Serialize)]
pub struct PkgMetadata {
    pkg: Pkg<'static>,
}

#[derive(Debug, Serialize)]
pub struct Pkg<'tcx> {
    types: FxHashMap<DefId, Ty<'tcx>>,
}

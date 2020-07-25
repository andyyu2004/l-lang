use crate::ir;
use crate::tir;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
crate struct Prog<'tcx> {
    pub items: BTreeMap<ir::Id, &'tcx tir::Item<'tcx>>,
}

impl<'tcx> Display for tir::Prog<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        tir::Formatter::new(f).fmt(self)
    }
}

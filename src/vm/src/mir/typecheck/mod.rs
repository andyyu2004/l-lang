use crate::mir::{self, *};
use crate::typeck::inference::InferCtx;

struct Typechecker<'a, 'tcx> {
    infcx: &'a InferCtx<'a, 'tcx>,
    mir: &'a mir::Mir<'tcx>,
}

impl<'a, 'tcx> Typechecker<'a, 'tcx> {
    pub fn typecheck(&mut self) {
        // self.visit_mir(&self.mir);
    }
}

pub fn check<'a, 'tcx>(mir: &'a mir::Mir<'tcx>, infcx: &InferCtx<'a, 'tcx>) {
    Typechecker { infcx, mir }.typecheck();
}

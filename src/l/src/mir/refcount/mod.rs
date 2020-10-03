//! analyses body and inserts drop calls
//! probably an undecidable problem

use crate::mir;
use crate::typeck::TyCtx;

pub trait MirPass<'tcx> {
    fn run_pass(&self, tcx: TyCtx<'tcx>, body: &mut mir::Mir<'tcx>);
}

struct RcCtx<'a, 'tcx> {
    mir: &'a mir::Mir<'tcx>,
}

impl<'a, 'tcx> RcCtx<'a, 'tcx> {
    pub fn new(mir: &'a mir::Mir<'tcx>) -> Self {
        Self { mir }
    }
}

impl<'tcx> MirPass<'tcx> for RcCtx<'_, 'tcx> {
    fn run_pass(&self, tcx: TyCtx<'tcx>, body: &mut mir::Mir<'tcx>) {
        todo!()
    }
}

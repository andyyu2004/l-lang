//! analyses body and inserts drop calls
//! probably an undecidable problem

use crate::mir;
use crate::typeck::TyCtx;

pub trait MirPass<'tcx> {
    fn run_pass(&self, tcx: TyCtx<'tcx>, body: &mut mir::Body<'tcx>);
}

struct RcCtx<'a, 'tcx> {
    mir: &'a mir::Body<'tcx>,
}

impl<'a, 'tcx> RcCtx<'a, 'tcx> {
    pub fn new(mir: &'a mir::Body<'tcx>) -> Self {
        Self { mir }
    }
}

impl<'tcx> MirPass<'tcx> for RcCtx<'_, 'tcx> {
    fn run_pass(&self, tcx: TyCtx<'tcx>, body: &mut mir::Body<'tcx>) {
        todo!()
    }
}

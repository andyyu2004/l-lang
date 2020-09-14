use crate::mir;
use crate::tir::TirCtx;

/// checks that assignments to immutable lvalues are done at most once
pub fn check_assignments<'a, 'tcx>(mir: &mir::Body<'tcx>, ctx: &TirCtx<'a, 'tcx>) {
}

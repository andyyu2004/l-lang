use crate::mir;
use crate::tir::TirCtx;

/// checks that assignments to immutable lvalues are done at most once
pub fn check_assignments<'a, 'tcx>(mir: &mir::Mir<'tcx>, ctx: &TirCtx<'a, 'tcx>) {
}

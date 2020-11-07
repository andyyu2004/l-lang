//! Sometimes, the pull-based query system is not sufficient to compile all programs correctly
//! consider an incorrect program such as the following
//!
//! struct S<T> {
//!     s: &S,
//! }
//!
//! fn main() -> int { 0 }
//!
//! Clearly, this should not compile as the field `s` has an incorrect number of type parameters
//! for the type `S`.
//!
//! However, using queries alone to compile will not catch this error as `S` is never referenced
//! from any function.
//!
//! One solution to this is to run some passes that will force everything to be checked, even if
//! never used.

use lcore::queries::Queries;
use lcore::TyCtx;

pub fn provide(queries: &mut Queries) {
    *queries = Queries { analyze: |tcx, ()| analyze(tcx), ..*queries }
}

/// runs all phases of analyses using the api queries provide
/// if no errors are caught during this, then the code should be correct
/// and safe to codegen
fn analyze<'tcx>(tcx: TyCtx<'tcx>) {
    // TODO
    // collect -> validate -> check
    PassRunner { tcx }.run_passes(&[&ItemTypeValidationPass]);
}

struct PassRunner<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> PassManager<'tcx> for PassRunner<'tcx> {
    fn run_passes(&mut self, passes: &[&dyn AnalysisPass<'tcx>]) {
        let tcx = self.tcx;
        for pass in passes {
            tcx.sess.prof.time(pass.name(), || pass.run_pass(tcx))
        }
    }
}

trait PassManager<'tcx> {
    fn run_passes(&mut self, passes: &[&dyn AnalysisPass<'tcx>]);
}

trait AnalysisPass<'tcx> {
    fn name(&self) -> &'static str;
    fn run_pass(&self, tcx: TyCtx<'tcx>);
}

struct ItemTypeCollectionPass;

impl<'tcx> AnalysisPass<'tcx> for ItemTypeCollectionPass {
    fn name(&self) -> &'static str {
        "item type collection pass"
    }

    fn run_pass(&self, tcx: TyCtx<'tcx>) {
        for item in tcx.ir.items.values() {
            tcx.validate_item_type(item.id.def);
        }
    }
}

struct ItemTypeValidationPass;

impl<'tcx> AnalysisPass<'tcx> for ItemTypeValidationPass {
    fn name(&self) -> &'static str {
        "item type validation pass"
    }

    fn run_pass(&self, tcx: TyCtx<'tcx>) {
        for item in tcx.ir.items.values() {
            tcx.validate_item_type(item.id.def);
        }
    }
}

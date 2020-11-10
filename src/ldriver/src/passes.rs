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
//!
//!
//!
//!

use ir::{FnVisitor, ItemVisitor};
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
    PassRunner { tcx }.run_passes(&mut [
        &mut ItemTypeCollectionPass { tcx },
        &mut ItemTypeValidationPass { tcx },
        &mut TypecheckPass { tcx },
    ]);
}

struct PassRunner<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> PassManager<'tcx> for PassRunner<'tcx> {
    fn run_passes(&mut self, passes: &mut [&mut dyn AnalysisPass<'tcx>]) {
        let tcx = self.tcx;
        for pass in passes {
            tcx.sess.prof.time(pass.name(), || pass.run_pass())
        }
    }
}

trait PassManager<'tcx> {
    fn run_passes(&mut self, passes: &mut [&mut dyn AnalysisPass<'tcx>]);
}

trait AnalysisPass<'tcx> {
    fn name(&self) -> &'static str;
    fn run_pass(&mut self);
}

struct ItemTypeCollectionPass<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> AnalysisPass<'tcx> for ItemTypeCollectionPass<'tcx> {
    fn name(&self) -> &'static str {
        "item type collection pass"
    }

    fn run_pass(&mut self) {
        for item in self.tcx.ir.items.values() {
            self.tcx.validate_item_type(item.id.def);
        }
    }
}

struct ItemTypeValidationPass<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> AnalysisPass<'tcx> for ItemTypeValidationPass<'tcx> {
    fn name(&self) -> &'static str {
        "item type validation pass"
    }

    fn run_pass(&mut self) {
        for item in self.tcx.ir.items.values() {
            self.tcx.validate_item_type(item.id.def);
        }
    }
}

/// typecheck function bodies
struct TypecheckPass<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> AnalysisPass<'tcx> for TypecheckPass<'tcx> {
    fn name(&self) -> &'static str {
        "typecheck pass"
    }

    fn run_pass(&mut self) {
        self.visit_ir(self.tcx.ir)
    }
}

impl<'tcx> FnVisitor<'tcx> for TypecheckPass<'tcx> {
    fn visit_fn(&mut self, def_id: ir::DefId) {
        let _ = self.tcx.typeck(def_id);
    }
}

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

use ir::{FnVisitor, ItemVisitor};
use lcore::queries::Queries;
use lcore::TyCtx;

pub fn provide(queries: &mut Queries) {
    *queries = Queries { analyze: |tcx, ()| analyze(tcx), ..*queries }
}

/// runs all phases of analyses using the api that the query system provides
/// if no errors are caught during these analyses, then the code should be correct
/// and safe to codegen
fn analyze<'tcx>(tcx: TyCtx<'tcx>) {
    tcx.sess.prof.time("analysis", || {
        PassRunner { tcx }.run_passes(&mut [
            &mut ItemTypeCollectionPass { tcx },
            &mut ItemTypeValidationPass { tcx },
            &mut TypecheckPass { tcx },
            &mut MirLoweringPass { tcx },
        ])
    })
}

struct PassRunner<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> PassManager<'tcx> for PassRunner<'tcx> {
    fn run_passes(&mut self, passes: &mut [&mut dyn AnalysisPass<'tcx>]) {
        let tcx = self.tcx;
        for pass in passes {
            if tcx.sess.try_run(|| tcx.sess.prof.time(pass.name(), || pass.run_pass())) == Err(true)
            {
                return;
            }
        }
    }
}

trait PassManager<'tcx> {
    fn run_passes(&mut self, passes: &mut [&mut dyn AnalysisPass<'tcx>]);
}

trait AnalysisPass<'tcx> {
    fn name(&self) -> &'static str;

    /// returns whether we should halt on any errors
    // try to avoid halting if possible
    fn run_pass(&mut self) -> bool;
}

struct ItemTypeCollectionPass<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> AnalysisPass<'tcx> for ItemTypeCollectionPass<'tcx> {
    fn name(&self) -> &'static str {
        "item type collection pass"
    }

    fn run_pass(&mut self) -> bool {
        for item in self.tcx.ir.items.values() {
            self.tcx.validate_item_type(item.id.def);
        }
        false
    }
}

struct ItemTypeValidationPass<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> AnalysisPass<'tcx> for ItemTypeValidationPass<'tcx> {
    fn name(&self) -> &'static str {
        "item type validation pass"
    }

    fn run_pass(&mut self) -> bool {
        for item in self.tcx.ir.items.values() {
            self.tcx.validate_item_type(item.id.def);
        }
        // TODO currently required to run some validation on impls, maybe can move elsewhere
        self.tcx.inherent_impls(());
        false
    }
}

impl_body_check_pass!(TypecheckPass, tcx, "type check pass", typeck, true);
impl_body_check_pass!(MirLoweringPass, tcx, "mir lowering pass", mir_of, true);

macro impl_body_check_pass($type:ident, $tcx:ident, $name:literal, $fn:ident, $halt_on_failure:expr) {
    struct $type<'tcx> {
        $tcx: TyCtx<'tcx>,
    }

    impl<'tcx> AnalysisPass<'tcx> for $type<'tcx> {
        fn name(&self) -> &'static str {
            $name
        }

        fn run_pass(&mut self) -> bool {
            self.visit_ir(self.$tcx.ir);
            $halt_on_failure
        }
    }

    impl<'tcx> FnVisitor<'tcx> for $type<'tcx> {
        fn visit_fn(&mut self, def_id: ir::DefId) {
            self.$tcx.$fn(def_id);
        }
    }
}

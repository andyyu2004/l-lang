//! this pass goes over the entire ir and constructs `TypeckTables` which resolves all inference
//! variables with their actual values

use crate::FnCtx;
use lcore::ty::{InferVarSubstsFolder, TypeFoldable, TypeckTables};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// constructs a new typeck table with all inference variables replaced by their actual types
    pub fn resolve_inference_variables(
        &self,
        _body: &'tcx ir::Body<'tcx>,
    ) -> &'tcx TypeckTables<'tcx> {
        let wbctx = WritebackCtx::new(self);
        self.tcx.arena.alloc(wbctx.tables)
    }
}

struct WritebackCtx<'a, 'tcx> {
    #[allow(unused)]
    fcx: &'a FnCtx<'a, 'tcx>,
    tables: TypeckTables<'tcx>,
}

impl<'a, 'tcx> WritebackCtx<'a, 'tcx> {
    fn new(fcx: &'a FnCtx<'a, 'tcx>) -> Self {
        let substs = fcx.inference_substs();
        let mut subst_folder = InferVarSubstsFolder::new(fcx.tcx, substs);

        let tables = fcx.tables.borrow();
        let tables = tables.fold_with(&mut subst_folder);
        Self { fcx, tables }
    }
}

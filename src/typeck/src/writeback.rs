//! this pass goes over the entire ir and constructs `TypeckTables` which resolves all inference
//! variables with their actual values

use crate::FnCtx;
use ir::{self, Visitor};
use lcore::ty::{InferenceVarSubstFolder, Ty, TypeFoldable, TypeckTables};
use span::Span;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// constructs a new typeck table with all inference variables replaced by their actual types
    pub fn resolve_inference_variables(
        &self,
        body: &'tcx ir::Body<'tcx>,
    ) -> &'tcx TypeckTables<'tcx> {
        let mut wbctx = WritebackCtx::new(self);
        wbctx.visit_body(body);
        self.tcx.arena.alloc(wbctx.tables)
    }
}

struct WritebackCtx<'a, 'tcx> {
    fcx: &'a FnCtx<'a, 'tcx>,
    tables: TypeckTables<'tcx>,
    subst_folder: InferenceVarSubstFolder<'tcx>,
}

impl<'a, 'tcx> WritebackCtx<'a, 'tcx> {
    fn new(fcx: &'a FnCtx<'a, 'tcx>) -> Self {
        // the `DefId` of the body is the same as the `DefId` of the expr of the body
        let substs = fcx.inference_substs();
        let subst_folder = InferenceVarSubstFolder::new(fcx.tcx, substs);

        // just clone as most of the stuff is the same
        // currently only node_types needs to be overwritten
        let mut tables = fcx.tables.borrow().clone();
        // we clear the `node_types` as we wish to overwrite them with the resolved types
        tables.node_types_mut().clear();
        Self { fcx, tables, subst_folder }
    }

    fn write_ty(&mut self, id: ir::Id, ty: Ty<'tcx>) {
        self.tables.node_types_mut().insert(id, ty);
    }
}

impl<'a, 'tcx> ir::Visitor<'tcx> for WritebackCtx<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx ir::Expr<'tcx>) {
        self.write_node_ty(expr.span, expr.id);
        ir::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &'tcx ir::Pattern<'tcx>) {
        self.write_node_ty(pat.span, pat.id);
        ir::walk_pat(self, pat);
    }
}

impl<'a, 'tcx> WritebackCtx<'a, 'tcx> {
    fn write_node_ty(&mut self, span: Span, id: ir::Id) {
        let unresolved_ty = self.fcx.node_ty(id);
        let ty = self.resolve_ty(unresolved_ty, span);
        self.write_ty(id, ty)
    }

    /// substitutes all the inference variables with their concrete types
    fn resolve_ty(&mut self, ty: Ty<'tcx>, _span: Span) -> Ty<'tcx> {
        ty.fold_with(&mut self.subst_folder)
    }
}

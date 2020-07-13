//! this pass goes over the entire hir and constructs a `TypeckTable` which replaces all inference
//! variables with their actual values

use super::{inference::FnCtx, TypeFoldable, TypeckTables};
use crate::ty::{InferenceVarSubstFolder, Ty};
use crate::{ir, span::Span};
use ir::Visitor;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn resolve_inference_variables(
        &self,
        body: &'tcx ir::Body<'tcx>,
    ) -> &'tcx TypeckTables<'tcx> {
        let mut wbctx = WritebackCtx::new(self, body);
        wbctx.visit_body(body);
        self.tcx.arena.alloc(wbctx.tables)
    }
}

struct WritebackCtx<'a, 'tcx> {
    fcx: &'a FnCtx<'a, 'tcx>,
    tables: TypeckTables<'tcx>,
    body: &'tcx ir::Body<'tcx>,
}

impl<'a, 'tcx> WritebackCtx<'a, 'tcx> {
    fn new(fcx: &'a FnCtx<'a, 'tcx>, body: &'tcx ir::Body<'tcx>) -> Self {
        // the `DefId` of the body is the same as the `DefId` of the expr of the body
        let def_id = body.expr.id.def_id;
        Self { fcx, tables: TypeckTables::new(def_id), body }
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
        ir::walk_pat(self, pat);
        self.write_node_ty(pat.span, pat.id);
    }
}

impl<'a, 'tcx> WritebackCtx<'a, 'tcx> {
    fn write_node_ty(&mut self, span: Span, id: ir::Id) {
        let unresolved_ty = self.fcx.node_ty(id);
        let ty = self.resolve_ty(unresolved_ty, span);
        self.write_ty(id, ty)
    }

    /// substitutes all the inference variables with their concrete types
    fn resolve_ty(&mut self, ty: Ty<'tcx>, span: Span) -> Ty<'tcx> {
        let substs = self.fcx.inference_substs().expect("unresolved inference variables");
        let mut folder = InferenceVarSubstFolder::new(self.fcx.tcx, substs);
        ty.fold_with(&mut folder)
    }
}

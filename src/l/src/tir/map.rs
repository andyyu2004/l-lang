use crate::{ir, tir};
use rustc_hash::FxHashMap;

pub struct Map<'tcx> {
    expr_map: FxHashMap<ir::Id, &'tcx tir::Expr<'tcx>>,
}

impl<'tcx> Map<'tcx> {
    pub fn def_expr(&mut self, id: ir::Id, expr: &'tcx tir::Expr<'tcx>) {
        self.expr_map.insert(id, expr);
    }

    pub fn expr(&mut self, id: ir::Id) -> &'tcx tir::Expr<'tcx> {
        self.expr_map[&id]
    }
}

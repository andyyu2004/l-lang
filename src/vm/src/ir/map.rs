use crate::ir;
use rustc_hash::FxHashMap;

pub struct IRMap<'ir> {
    expr_map: FxHashMap<ir::Id, &'ir ir::Expr<'ir>>,
}

impl<'ir> IRMap<'ir> {
    pub fn def_expr(&mut self, id: ir::Id, expr: &'ir ir::Expr<'ir>) {
        self.expr_map.insert(id, expr);
    }

    pub fn expr(&mut self, id: ir::Id) -> &'ir ir::Expr<'ir> {
        self.expr_map[&id]
    }
}

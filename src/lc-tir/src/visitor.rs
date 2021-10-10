use crate as tir;

/// tir Visitor
pub trait Visitor<'tcx> {
    fn visit_expr(&mut self, expr: &'tcx tir::Expr<'tcx>) -> bool;
    fn visit_item(&mut self, item: &'tcx tir::Item<'tcx>) -> bool;
}

/// tir Folder
pub trait Folder<'tcx>: Sized {
    fn fold_expr(&mut self, expr: &'tcx tir::Expr<'tcx>) -> &'tcx tir::Expr<'tcx>;
    fn fold_item(&mut self, item: &'tcx tir::Item<'tcx>) -> &'tcx tir::Item<'tcx>;
}

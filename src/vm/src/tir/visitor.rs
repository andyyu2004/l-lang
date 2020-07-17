use crate::tir;
use crate::ty::{TypeFoldable, TypeFolder, TypeVisitor};

/// tir Visitor
crate trait Visitor<'tcx> {
    fn visit_expr(&mut self, expr: &'tcx tir::Expr<'tcx>) -> bool;
    fn visit_item(&mut self, item: &'tcx tir::Item<'tcx>) -> bool;
}

/// tir Folder
crate trait Folder<'tcx>: Sized {
    fn fold_expr(&mut self, expr: &'tcx tir::Expr<'tcx>) -> &'tcx tir::Expr<'tcx>;
    fn fold_item(&mut self, item: &'tcx tir::Item<'tcx>) -> &'tcx tir::Item<'tcx>;
}

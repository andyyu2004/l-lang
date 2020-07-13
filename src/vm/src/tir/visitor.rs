use crate::tir;
use crate::typeck::{TypeFoldable, TypeFolder, TypeVisitor};

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

impl<'tcx, T> TypeFoldable<'tcx> for &'tcx [T]
where
    T: TypeFoldable<'tcx>,
{
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        folder.tcx().alloc_tir_iter(self.iter().map(|t| t.fold_with(folder)))
    }

    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        todo!()
    }
}

impl<'tcx> TypeFoldable<'tcx> for &'tcx tir::Stmt<'tcx> {
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        todo!()
    }
    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        todo!()
    }
}

impl<'tcx> TypeFoldable<'tcx> for &'tcx tir::Block<'tcx> {
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        todo!()
    }

    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        todo!()
    }
}

impl<'tcx> TypeFoldable<'tcx> for &'tcx tir::Expr<'tcx> {
    fn inner_fold_with<F>(&self, f: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        let kind = match &self.kind {
            tir::ExprKind::Lit(_) => return self,
            &tir::ExprKind::Bin(op, l, r) => tir::ExprKind::Bin(op, l.fold_with(f), r.fold_with(f)),
            &tir::ExprKind::Unary(op, expr) => tir::ExprKind::Unary(op, expr.fold_with(f)),
            tir::ExprKind::Block(block) => tir::ExprKind::Block(block.fold_with(f)),
        };

        let ty = self.ty.fold_with(f);
        f.tcx().alloc_tir(tir::Expr { span: self.span, kind, ty })
    }

    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        todo!()
    }
}

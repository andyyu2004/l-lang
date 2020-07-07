use super::TyCtx;
use crate::ty::{Ty, TyKind};

crate trait TypeFoldable<'tcx>: Sized {
    /// recursively fold inner `Ty<'tcx>`s
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>;

    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>;

    fn fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        self.inner_fold_with(folder)
    }

    fn visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        self.inner_visit_with(visitor)
    }
}

impl<'tcx> TypeFoldable<'tcx> for Ty<'tcx> {
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        let kind = match self.kind {
            TyKind::Infer(_)
            | TyKind::Unit
            | TyKind::Char
            | TyKind::Num
            | TyKind::Bool
            | TyKind::_Phantom(_) => {
                return self;
            }
        };

        if kind == self.kind { self } else { folder.tcx().mk_ty(kind) }
    }

    fn fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        folder.fold_ty(self)
    }

    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        match self.kind {
            TyKind::Infer(_) => todo!(),
            TyKind::Unit | TyKind::Char | TyKind::Num | TyKind::Bool => return false,
            TyKind::_Phantom(_) => false,
        }
    }
}

crate trait TypeFolder<'tcx>: Sized {
    fn tcx(&self) -> TyCtx<'tcx>;
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        ty.fold_with(self)
    }
}

crate trait TypeVisitor<'tcx>: Sized {
    fn visit_ty(&mut self, ty: Ty<'tcx>) -> bool {
        ty.visit_with(self)
    }
}

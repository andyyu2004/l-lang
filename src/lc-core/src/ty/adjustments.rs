use crate::ty::{Ty, TypeFoldable, TypeFolder, TypeVisitor};

#[derive(Debug, Clone, Copy)]
pub struct Adjustment<'tcx> {
    pub ty: Ty<'tcx>,
    pub kind: AdjustmentKind,
}

impl<'tcx> Adjustment<'tcx> {
    pub fn new(ty: Ty<'tcx>, kind: AdjustmentKind) -> Self {
        Self { ty, kind }
    }

    pub fn new_deref(ty: Ty<'tcx>) -> Self {
        Self::new(ty, AdjustmentKind::Deref)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerCast {
    /// from fndef to fnptr
    ReifyFn,
}

#[derive(Debug, Clone, Copy)]
pub enum AdjustmentKind {
    Deref,
    NeverToAny,
    Cast(PointerCast),
}

pub trait Adjuster<'tcx> {
    fn get_adjustments(&self) -> Vec<Adjustment<'tcx>>;
}

impl<'tcx> TypeFoldable<'tcx> for Adjustment<'tcx> {
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        Self { ty: self.ty.fold_with(folder), kind: self.kind }
    }

    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        self.ty.visit_with(visitor)
    }
}

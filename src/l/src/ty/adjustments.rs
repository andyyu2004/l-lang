use crate::ty::Ty;

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

#[derive(Debug, Clone, Copy)]
pub enum AdjustmentKind {
    Deref,
    NeverToAny,
}

pub trait Adjuster<'tcx> {
    fn get_adjustments(&self) -> Vec<Adjustment<'tcx>>;
}

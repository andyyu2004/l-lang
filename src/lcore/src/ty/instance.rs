use crate::ty::{Subst, SubstsRef, Ty, TyCtx};
use ir::DefId;
use std::fmt::{self, Display, Formatter};

/// a generic definition along with its concrete substitutions
/// used for monomorphization
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Instance<'tcx> {
    pub substs: SubstsRef<'tcx>,
    pub kind: InstanceDef,
}

impl<'tcx> Instance<'tcx> {
    pub fn ty(self, tcx: TyCtx<'tcx>) -> Ty<'tcx> {
        let ty = self.kind.ty(tcx);
        ty.subst(tcx, self.substs)
    }

    /// construct a new instance of an item
    pub fn item(substs: SubstsRef<'tcx>, def_id: DefId) -> Self {
        Instance { substs, kind: InstanceDef::Item(def_id) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InstanceDef {
    Item(DefId),
}

impl InstanceDef {
    fn ty<'tcx>(self, tcx: TyCtx<'tcx>) -> Ty<'tcx> {
        match self {
            InstanceDef::Item(def_id) => tcx.collected_ty(def_id),
        }
    }
}

impl<'tcx> Display for Instance<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}<{}>", self.kind, self.substs)
    }
}

impl Display for InstanceDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InstanceDef::Item(def_id) => write!(f, "{}", def_id),
        }
    }
}

use crate::ty::{Subst, Substs, SubstsRef, Ty, TyCtx};
use ir::DefId;
use std::fmt::{self, Display, Formatter};

/// a generic definition along with its concrete substitutions
/// used for monomorphization
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Instance<'tcx> {
    pub substs: SubstsRef<'tcx>,
    pub def_id: DefId,
    pub kind: InstanceKind,
}

impl<'tcx> Instance<'tcx> {
    pub fn ty(self, tcx: TyCtx<'tcx>) -> Ty<'tcx> {
        let ty = match self.kind {
            InstanceKind::Item => tcx.collected_ty(self.def_id),
        };
        ty.subst(tcx, self.substs)
    }

    /// construct a new instance of an item
    pub fn item(substs: SubstsRef<'tcx>, def_id: DefId) -> Self {
        Instance { substs, def_id, kind: InstanceKind::Item }
    }

    pub fn mono_item(def_id: DefId) -> Self {
        Self::item(Substs::empty(), def_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InstanceKind {
    // maybe include the mir body in here?
    Item,
}

impl<'tcx> Display for Instance<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            InstanceKind::Item => write!(f, "{}<{}>", self.def_id, self.substs),
        }
    }
}

use crate::ty::{Subst, Substs, SubstsRef, Ty, TyCtx};
use ir::DefId;
use std::fmt::{self, Display, Formatter};

index::newtype_index! {
    pub struct InstanceId {
        DEBUG_FORMAT = "{}"
    }
}

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
        let ty = tcx.collected_ty(self.def_id);
        ty.subst(tcx, self.substs)
    }

    pub fn resolve(tcx: TyCtx<'tcx>, def_id: DefId, substs: SubstsRef<'tcx>) -> Self {
        match tcx.defs().get(def_id) {
            // can just treat constructors as normal items
            ir::DefNode::Item(..) | ir::DefNode::Ctor(..) => Instance::item(def_id, substs),
            ir::DefNode::ForeignItem(..) => Instance::intrinsic(def_id, substs),
            ir::DefNode::ImplItem(..) => todo!(),
            ir::DefNode::Variant(..) => todo!(),
        }
    }

    /// construct a new instance of an item
    pub fn item(def_id: DefId, substs: SubstsRef<'tcx>) -> Self {
        Instance { substs, def_id, kind: InstanceKind::Item }
    }

    pub fn intrinsic(def_id: DefId, substs: SubstsRef<'tcx>) -> Self {
        Instance { substs, def_id, kind: InstanceKind::Intrinsic }
    }

    pub fn mono_item(def_id: DefId) -> Self {
        Self::item(def_id, Substs::empty())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InstanceKind {
    // maybe include the mir body in here?
    Item,
    Intrinsic,
}

impl<'tcx> Display for Instance<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}<{}>", self.def_id, self.substs)
    }
}

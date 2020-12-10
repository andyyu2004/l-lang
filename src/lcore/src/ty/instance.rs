use crate::queries::Queries;
use crate::ty::{Subst, Substs, SubstsRef, Ty, TyCtx};
use ast::Abi;
use ir::DefId;
use rustc_hash::FxHashSet;
use std::fmt::{self, Display, Formatter};

crate fn provide(queries: &mut Queries) {
    *queries = Queries { resolve_instance, ..*queries }
}

fn resolve_instance<'tcx>(
    tcx: TyCtx<'tcx>,
    (def_id, substs): (DefId, SubstsRef<'tcx>),
) -> Instance<'tcx> {
    match tcx.defs().get(def_id) {
        // can just treat constructors as normal items
        ir::DefNode::Item(..) | ir::DefNode::ImplItem(..) | ir::DefNode::Ctor(..) =>
            Instance::item(def_id, substs),
        ir::DefNode::ForeignItem(item) if item.abi == Abi::Intrinsic =>
            Instance::intrinsic(def_id, substs),
        ir::DefNode::ForeignItem(_) => todo!(),
        ir::DefNode::Field(..) | ir::DefNode::Variant(..) | ir::DefNode::TyParam(..) =>
            unreachable!(),
    }
}

pub type Instances<'tcx> = FxHashSet<Instance<'tcx>>;

/// a generic definition along with its concrete substitutions
/// represents an `instance` of monomorphization
/// i.e. a generic function maybe monomorphized/instantiated into multiple instances
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Instance<'tcx> {
    pub def_id: DefId,
    pub substs: SubstsRef<'tcx>,
    pub kind: InstanceKind,
}

impl<'tcx> Instance<'tcx> {
    pub fn ty(self, tcx: TyCtx<'tcx>) -> Ty<'tcx> {
        let ty = tcx.type_of(self.def_id);
        ty.subst(tcx, self.substs)
    }

    pub fn resolve(tcx: TyCtx<'tcx>, def_id: DefId, substs: SubstsRef<'tcx>) -> Self {
        tcx.resolve_instance((def_id, substs))
    }

    /// construct a new instance of an item
    fn item(def_id: DefId, substs: SubstsRef<'tcx>) -> Self {
        Instance { substs, def_id, kind: InstanceKind::Item }
    }

    fn intrinsic(def_id: DefId, substs: SubstsRef<'tcx>) -> Self {
        Instance { substs, def_id, kind: InstanceKind::Intrinsic }
    }

    pub fn mono_item(def_id: DefId) -> Self {
        Self::item(def_id, Substs::empty())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InstanceKind {
    Item,
    Intrinsic,
}

impl<'tcx> Display for Instance<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}<{}>", self.def_id, self.substs)
    }
}

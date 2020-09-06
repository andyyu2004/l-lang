use crate::ir::{self, DefId, LocalId};
use crate::ty::Ty;
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct TypeckOutputs<'tcx> {
    /// the `DefId` that the `LocalId`s in this table are relative to
    pub def_id: DefId,
    pub node_types: FxHashMap<LocalId, Ty<'tcx>>,
}

impl<'tcx> TypeckOutputs<'tcx> {
    pub fn new(def_id: DefId) -> Self {
        Self { def_id, node_types: Default::default() }
    }

    pub fn node_type(&self, id: ir::Id) -> Ty<'tcx> {
        self.node_type_opt(id).expect(&format!("no entry for node `{}` in typecktables", id))
    }

    pub fn node_type_opt(&self, id: ir::Id) -> Option<Ty<'tcx>> {
        self.node_types().get(id).cloned()
    }

    pub fn node_types(&self) -> TableDefIdValidator<Ty<'tcx>> {
        TableDefIdValidator { def_id: self.def_id, table: &self.node_types }
    }

    pub fn node_types_mut(&mut self) -> TableDefIdValidatorMut<Ty<'tcx>> {
        TableDefIdValidatorMut { def_id: self.def_id, table: &mut self.node_types }
    }
}

pub struct TableDefIdValidator<'a, T> {
    def_id: DefId,
    table: &'a FxHashMap<LocalId, T>,
}

impl<'a, T> TableDefIdValidator<'a, T> {
    fn validate_id(&self, id: ir::Id) {
        assert_eq!(self.def_id, id.def);
    }

    pub fn get(&self, id: ir::Id) -> Option<&T> {
        validate_id(self.def_id, id);
        self.table.get(&id.local)
    }
}

pub struct TableDefIdValidatorMut<'a, T> {
    def_id: DefId,
    table: &'a mut FxHashMap<LocalId, T>,
}

fn validate_id(def_id: DefId, id: ir::Id) {
    assert_eq!(def_id, id.def);
}

impl<'a, T> TableDefIdValidatorMut<'a, T> {
    pub fn insert(&mut self, id: ir::Id, value: T) -> Option<T> {
        validate_id(self.def_id, id);
        self.table.insert(id.local, value)
    }
}

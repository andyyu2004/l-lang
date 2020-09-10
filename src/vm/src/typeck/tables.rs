use crate::ir::{self, DefId, FieldIdx, LocalId};
use crate::ty::Ty;
use rustc_hash::FxHashMap;

/// the outputs of typechecking
#[derive(Debug)]
pub struct TypeckOutputs<'tcx> {
    /// the `DefId` that the `LocalId`s in this table are relative to
    pub def_id: DefId,
    pub node_types: FxHashMap<LocalId, Ty<'tcx>>,
    /// the index within a struct a field is assigned
    pub field_indices: FxHashMap<LocalId, FieldIdx>,
}

impl<'tcx> TypeckOutputs<'tcx> {
    pub fn new(def_id: DefId) -> Self {
        Self { def_id, node_types: Default::default(), field_indices: Default::default() }
    }

    pub fn node_type(&self, id: ir::Id) -> Ty<'tcx> {
        self.node_type_opt(id).expect(&format!("no entry for node `{}` in `node_types`", id))
    }

    pub fn field_index(&self, id: ir::Id) -> FieldIdx {
        self.field_index_opt(id).expect(&format!("no entry for `{}` in `field_indices`", id))
    }

    pub fn field_index_opt(&self, id: ir::Id) -> Option<FieldIdx> {
        self.field_indices().get(id).copied()
    }

    pub fn node_type_opt(&self, id: ir::Id) -> Option<Ty<'tcx>> {
        self.node_types().get(id).copied()
    }

    pub fn node_types(&self) -> TableDefIdValidator<Ty<'tcx>> {
        TableDefIdValidator { def_id: self.def_id, table: &self.node_types }
    }

    pub fn node_types_mut(&mut self) -> TableDefIdValidatorMut<Ty<'tcx>> {
        TableDefIdValidatorMut { def_id: self.def_id, table: &mut self.node_types }
    }

    pub fn field_indices(&self) -> TableDefIdValidator<FieldIdx> {
        TableDefIdValidator { def_id: self.def_id, table: &self.field_indices }
    }

    pub fn field_indices_mut(&mut self) -> TableDefIdValidatorMut<FieldIdx> {
        TableDefIdValidatorMut { def_id: self.def_id, table: &mut self.field_indices }
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

    pub fn remove(&mut self, id: ir::Id) -> Option<T> {
        validate_id(self.def_id, id);
        self.table.remove(&id.local)
    }
}

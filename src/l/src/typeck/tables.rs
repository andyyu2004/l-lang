use crate::ir::{self, DefId, FieldIdx, LocalId};
use crate::ty::Adjustment;
use crate::ty::{Ty, UpvarId};
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::hash_map::Entry;

/// the outputs of typechecking
#[derive(Debug, Clone)]
pub struct TypeckTables<'tcx> {
    /// the `DefId` that the `LocalId`s in this table are relative to
    def_id: DefId,
    node_types: FxHashMap<LocalId, Ty<'tcx>>,
    /// the index within a struct a field is assigned
    field_indices: FxHashMap<LocalId, FieldIdx>,
    adjustments: FxHashMap<LocalId, Vec<Adjustment<'tcx>>>,
    upvar_captures: FxHashMap<ir::Id, FxHashSet<UpvarId>>,
}

impl<'tcx> TypeckTables<'tcx> {
    pub fn new(def_id: DefId) -> Self {
        Self {
            def_id,
            node_types: Default::default(),
            adjustments: Default::default(),
            field_indices: Default::default(),
            upvar_captures: Default::default(),
        }
    }

    pub fn upvar_captures_for_closure(&self, closure_id: ir::Id) -> &FxHashSet<UpvarId> {
        &self.upvar_captures[&closure_id]
    }

    pub fn record_upvar_capture_for_closure(
        &mut self,
        closure_id: ir::Id,
        upvars: FxHashSet<UpvarId>,
    ) {
        match self.upvar_captures.entry(closure_id) {
            Entry::Vacant(entry) => entry.insert(upvars),
            Entry::Occupied(_) => panic!("upvars already set for closure `{}`", closure_id),
        };
    }

    pub fn record_upvar_capture(&mut self, upvar: UpvarId) {
        if self.upvar_captures.entry(upvar.closure_id).or_default().insert(upvar) {
            panic!("variable captured twice by the same closure")
        }
    }

    pub fn node_type(&self, id: ir::Id) -> Ty<'tcx> {
        self.node_type_opt(id)
            .unwrap_or_else(|| panic!("no entry for node `{}` in `node_types`", id))
    }

    pub fn field_index(&self, id: ir::Id) -> FieldIdx {
        self.field_index_opt(id)
            .unwrap_or_else(|| panic!("no entry for `{}` in `field_indices`", id))
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

    pub fn adjustments_for_expr(&self, expr: &ir::Expr) -> &[Adjustment<'tcx>] {
        // can't use `self.adjustments()` due to lifetime issues
        validate_id(self.def_id, expr.id);
        self.adjustments.get(&expr.id.local).map_or(&[], |xs| &xs[..])
    }

    pub fn adjustments(&self) -> TableDefIdValidator<Vec<Adjustment<'tcx>>> {
        TableDefIdValidator { def_id: self.def_id, table: &self.adjustments }
    }

    pub fn adjustments_mut(&mut self) -> TableDefIdValidatorMut<Vec<Adjustment<'tcx>>> {
        TableDefIdValidatorMut { def_id: self.def_id, table: &mut self.adjustments }
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

    pub fn clear(&mut self) {
        self.table.clear()
    }
}

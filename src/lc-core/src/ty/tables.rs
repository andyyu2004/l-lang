use crate::ty::{Adjustment, SubstsRef, Ty, TypeFoldable, TypeFolder, TypeVisitor, UpvarId};
use ir::{self, DefId, FieldIdx, LocalId, Res};
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::hash_map::Entry;

/// the outputs of typechecking
#[derive(Debug, Clone)]
pub struct TypeckTables<'tcx> {
    /// the `DefId` that the `LocalId`s in this table are relative to
    def_id: DefId,
    adjustments: FxHashMap<LocalId, Vec<Adjustment<'tcx>>>,
    node_types: FxHashMap<LocalId, Ty<'tcx>>,
    /// the substitutions applied to a node to obtain its type;
    /// this applies to generic objects (i.e. functions or adts)
    node_substs: FxHashMap<LocalId, SubstsRef<'tcx>>,
    /// the index within a struct a field is assigned
    field_indices: FxHashMap<LocalId, FieldIdx>,
    /// the resolution of a type relative path
    type_relative_resolutions: FxHashMap<LocalId, Res>,
    upvar_captures: FxHashMap<ir::Id, FxHashSet<UpvarId>>,
}

impl<'tcx> TypeckTables<'tcx> {
    pub fn new(def_id: DefId) -> Self {
        Self {
            def_id,
            node_types: Default::default(),
            node_substs: Default::default(),
            adjustments: Default::default(),
            field_indices: Default::default(),
            upvar_captures: Default::default(),
            type_relative_resolutions: Default::default(),
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

    pub fn node_substs_opt(&self, id: ir::Id) -> Option<SubstsRef<'tcx>> {
        self.node_substs().get(id).copied()
    }

    pub fn node_type_opt(&self, id: ir::Id) -> Option<Ty<'tcx>> {
        self.node_types().get(id).copied()
    }

    pub fn type_relative_res(&self, xpat: &dyn ir::ExprOrPat<'tcx>) -> Res {
        self.type_relative_resolutions().get(xpat.id()).copied().unwrap()
    }

    pub fn node_types(&self) -> TableDefIdValidator<'_, Ty<'tcx>> {
        TableDefIdValidator { def_id: self.def_id, table: &self.node_types }
    }

    pub fn node_types_mut(&mut self) -> TableDefIdValidatorMut<'_, Ty<'tcx>> {
        TableDefIdValidatorMut { def_id: self.def_id, table: &mut self.node_types }
    }

    pub fn node_substs(&self) -> TableDefIdValidator<'_, SubstsRef<'tcx>> {
        TableDefIdValidator { def_id: self.def_id, table: &self.node_substs }
    }

    pub fn node_substs_mut(&mut self) -> TableDefIdValidatorMut<'_, SubstsRef<'tcx>> {
        TableDefIdValidatorMut { def_id: self.def_id, table: &mut self.node_substs }
    }

    pub fn field_indices(&self) -> TableDefIdValidator<'_, FieldIdx> {
        TableDefIdValidator { def_id: self.def_id, table: &self.field_indices }
    }

    pub fn field_indices_mut(&mut self) -> TableDefIdValidatorMut<'_, FieldIdx> {
        TableDefIdValidatorMut { def_id: self.def_id, table: &mut self.field_indices }
    }

    pub fn adjustments_for_expr(&self, expr: &ir::Expr<'_>) -> &[Adjustment<'tcx>] {
        // can't use `self.adjustments()` due to lifetime issues
        assert_eq!(self.def_id, expr.id.def);
        self.adjustments.get(&expr.id.local).map_or(&[], |xs| &xs[..])
    }

    pub fn adjustments(&self) -> TableDefIdValidator<'_, Vec<Adjustment<'tcx>>> {
        TableDefIdValidator { def_id: self.def_id, table: &self.adjustments }
    }

    pub fn adjustments_mut(&mut self) -> TableDefIdValidatorMut<'_, Vec<Adjustment<'tcx>>> {
        TableDefIdValidatorMut { def_id: self.def_id, table: &mut self.adjustments }
    }

    pub fn type_relative_resolutions(&self) -> TableDefIdValidator<'_, Res> {
        TableDefIdValidator { def_id: self.def_id, table: &self.type_relative_resolutions }
    }

    pub fn type_relative_resolutions_mut(&mut self) -> TableDefIdValidatorMut<'_, Res> {
        TableDefIdValidatorMut { def_id: self.def_id, table: &mut self.type_relative_resolutions }
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
        self.validate_id(id);
        self.table.get(&id.local)
    }
}

pub struct TableDefIdValidatorMut<'a, T> {
    def_id: DefId,
    table: &'a mut FxHashMap<LocalId, T>,
}

impl<'a, T> TableDefIdValidatorMut<'a, T> {
    fn validate_id(&self, id: ir::Id) {
        assert_eq!(self.def_id, id.def);
    }

    pub fn insert(&mut self, id: ir::Id, value: T) -> Option<T> {
        self.validate_id(id);
        self.table.insert(id.local, value)
    }

    pub fn remove(&mut self, id: ir::Id) -> Option<T> {
        self.validate_id(id);
        self.table.remove(&id.local)
    }

    pub fn clear(&mut self) {
        self.table.clear()
    }
}

// this impl is used in writeback to substitute all inference variables with their final type
impl<'tcx> TypeFoldable<'tcx> for TypeckTables<'tcx> {
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        Self {
            def_id: self.def_id,
            adjustments: self.adjustments.iter().map(|(&k, v)| (k, v.fold_with(folder))).collect(),
            node_types: self.node_types.iter().map(|(&k, v)| (k, v.fold_with(folder))).collect(),
            node_substs: self.node_substs.iter().map(|(&k, v)| (k, v.fold_with(folder))).collect(),
            field_indices: self.field_indices.clone(),
            upvar_captures: self.upvar_captures.clone(),
            type_relative_resolutions: self.type_relative_resolutions.clone(),
        }
    }

    fn inner_visit_with<V>(&self, _visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        unimplemented!()
    }
}

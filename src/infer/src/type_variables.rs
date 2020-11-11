use crate::*;
use ena::snapshot_vec as sv;
use ena::undo_log::Rollback;
use ena::unify as ut;
use lcore::ty::{self, Ty, TyKind, TyVid, TypeError, TypeResult};
use rustc_hash::FxHashMap;
use span::Span;
use std::marker::PhantomData;

pub enum TyVarUndoLog<'tcx> {
    EqRelation(sv::UndoLog<ut::Delegate<TyVidEqKey<'tcx>>>),
    SubRelation(sv::UndoLog<ut::Delegate<ty::TyVid>>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TyVarValue<'tcx> {
    Known(Ty<'tcx>),
    Unknown,
}

#[derive(Debug)]
pub struct TyVarData {
    pub span: Span,
}

#[derive(Default, Debug)]
pub struct TypeVariableStorage<'tcx> {
    crate tyvar_data: FxHashMap<TyVid, TyVarData>,
    /// the number of type variables that have been generated
    eq_relations: ut::UnificationTableStorage<TyVidEqKey<'tcx>>,
}

impl<'tcx> Rollback<TyVarUndoLog<'tcx>> for TypeVariableStorage<'tcx> {
    fn reverse(&mut self, undo: TyVarUndoLog<'tcx>) {
        match undo {
            TyVarUndoLog::EqRelation(undo) => self.eq_relations.reverse(undo),
            TyVarUndoLog::SubRelation(_) => unimplemented!(),
        }
    }
}

impl<'tcx> TypeVariableStorage<'tcx> {
    crate fn with_log<'a>(
        &'a mut self,
        undo_log: &'a mut InferCtxUndoLogs<'tcx>,
    ) -> TypeVariableTable<'a, 'tcx> {
        TypeVariableTable { storage: self, undo_log }
    }
}

crate type UnificationTable<'a, 'tcx, T> = ut::UnificationTable<
    ut::InPlace<T, &'a mut ut::UnificationStorage<T>, &'a mut InferCtxUndoLogs<'tcx>>,
>;

pub struct TypeVariableTable<'a, 'tcx> {
    crate storage: &'a mut TypeVariableStorage<'tcx>,
    undo_log: &'a mut InferCtxUndoLogs<'tcx>,
}

impl<'a, 'tcx> TypeVariableTable<'a, 'tcx> {
    fn eq_relations(&mut self) -> UnificationTable<'_, 'tcx, TyVidEqKey<'tcx>> {
        self.storage.eq_relations.with_log(&mut self.undo_log)
    }

    /// if `ty` is known, return its known type, otherwise just return `t`
    pub fn instantiate_if_known(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        match ty.kind {
            TyKind::Infer(ty::TyVar(v)) => match self.probe(v) {
                TyVarValue::Known(t) => t,
                TyVarValue::Unknown => ty,
            },
            _ => ty,
        }
    }

    /// assumption that `vid` has not been instantiated before
    /// returns an error if the occurs check fails
    pub fn instantiate(&mut self, vid: TyVid, ty: Ty<'tcx>) -> TypeResult<'tcx, ()> {
        let root = self.root_var(vid);
        // there probably is a way to check this without searching the type
        // if this becomes a performance bottleneck we can have a look
        if ty.contains_tyvid(root) {
            Err(TypeError::OccursCheck(root, ty))
        } else {
            self.eq_relations().union_value(root, TyVarValue::Known(ty));
            Ok(())
        }
    }

    fn root_var(&mut self, vid: TyVid) -> TyVid {
        self.eq_relations().find(vid).vid
    }

    /// assumption that both `s` and `t` are unknown
    pub fn equate(&mut self, s: TyVid, t: TyVid) {
        self.eq_relations().union(s, t)
    }

    pub fn probe(&mut self, vid: TyVid) -> TyVarValue<'tcx> {
        self.eq_relations().probe_value(vid)
    }

    pub fn new_ty_var(&mut self, span: Span) -> TyVid {
        let mut tables = self.eq_relations();
        let key = tables.new_key(TyVarValue::Unknown);
        self.storage.tyvar_data.insert(key.vid, TyVarData { span });
        key.vid
    }
}

impl<'tcx> TyVarValue<'tcx> {
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl<'tcx> ut::UnifyValue for TyVarValue<'tcx> {
    type Error = ut::NoError;

    fn unify_values(s: &Self, t: &Self) -> Result<Self, Self::Error> {
        match (s, t) {
            (&Self::Known(_), &Self::Known(_)) => panic!("unifying two known type variables"),
            (&Self::Known(_), _) => Ok(*s),
            (_, &Self::Known(_)) => Ok(*t),
            (&Self::Unknown, &Self::Unknown) => Ok(TyVarValue::Unknown),
        }
    }
}

/// These structs (a newtyped TyVid) are used as the unification key
/// for the `eq_relations`; they carry a `TypeVariableValue` along
/// with them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TyVidEqKey<'tcx> {
    pub vid: TyVid,
    // in the table, we map each ty-vid to one of these:
    phantom: PhantomData<TyVarValue<'tcx>>,
}

impl<'tcx> From<TyVid> for TyVidEqKey<'tcx> {
    fn from(vid: TyVid) -> Self {
        TyVidEqKey { vid, phantom: PhantomData }
    }
}

impl<'tcx> ut::UnifyKey for TyVidEqKey<'tcx> {
    type Value = TyVarValue<'tcx>;

    fn index(&self) -> u32 {
        self.vid.index
    }

    fn from_index(i: u32) -> Self {
        TyVidEqKey::from(TyVid { index: i })
    }

    fn tag() -> &'static str {
        "TyVidEqKey"
    }
}

impl<'tcx> From<sv::UndoLog<ut::Delegate<TyVidEqKey<'tcx>>>> for TyVarUndoLog<'tcx> {
    fn from(l: sv::UndoLog<ut::Delegate<TyVidEqKey<'tcx>>>) -> Self {
        TyVarUndoLog::EqRelation(l)
    }
}

impl<'tcx> From<sv::UndoLog<ut::Delegate<ty::TyVid>>> for TyVarUndoLog<'tcx> {
    fn from(l: sv::UndoLog<ut::Delegate<ty::TyVid>>) -> Self {
        TyVarUndoLog::SubRelation(l)
    }
}

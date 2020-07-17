use super::InferCtxUndoLogs;
use crate::error::{TypeError, TypeResult};
use crate::ty::InferenceVarSubstFolder;
use crate::ty::{self, Ty, TyKind};
use ena::unify as ut;
use std::marker::PhantomData;

/// type variable id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
crate struct TyVid {
    pub index: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
crate enum TyVarValue<'tcx> {
    Known(Ty<'tcx>),
    Unknown,
}

#[derive(Default, Debug)]
crate struct TypeVariableStorage<'tcx> {
    eq_relations: ut::UnificationTableStorage<TyVidEqKey<'tcx>>,
    /// the number of type variables that have been generated
    tyvid_count: u32,
}

impl<'tcx> TypeVariableStorage<'tcx> {
    pub fn with_log<'a>(
        &'a mut self,
        undo_log: &'a mut InferCtxUndoLogs<'tcx>,
    ) -> TypeVariableTable<'a, 'tcx> {
        TypeVariableTable { storage: self, undo_log }
    }
}

pub(crate) type UnificationTable<'a, 'tcx, T> = ut::UnificationTable<
    ut::InPlace<T, &'a mut ut::UnificationStorage<T>, &'a mut InferCtxUndoLogs<'tcx>>,
>;

crate struct TypeVariableTable<'a, 'tcx> {
    storage: &'a mut TypeVariableStorage<'tcx>,
    undo_log: &'a mut InferCtxUndoLogs<'tcx>,
}

impl<'a, 'tcx> TypeVariableTable<'a, 'tcx> {
    fn eq_relations(&mut self) -> UnificationTable<'_, 'tcx, TyVidEqKey<'tcx>> {
        self.storage.eq_relations.with_log(&mut self.undo_log)
    }

    /// generates an indexed substitution based on the contents of the UnificationTable
    pub fn gen_substs(&mut self) -> TypeResult<'tcx, Vec<Ty<'tcx>>> {
        (0..self.storage.tyvid_count)
            .map(|index| self.probe(TyVid { index }))
            .map(|val| match val {
                TyVarValue::Known(ty) => Ok(ty),
                TyVarValue::Unknown => Err(TypeError::InferenceFailure),
            })
            .collect()
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
    pub fn instantiate(&mut self, vid: TyVid, ty: Ty<'tcx>) {
        let root = self.root_var(vid);
        self.eq_relations().union_value(root, TyVarValue::Known(ty))
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

    pub fn new_ty_var(&mut self) -> TyVid {
        let mut tables = self.eq_relations();
        let key = tables.new_key(TyVarValue::Unknown);
        debug_assert_eq!(key.vid.index, self.storage.tyvid_count);
        self.storage.tyvid_count += 1;
        key.vid
    }
}

impl<'tcx> TyVarValue<'tcx> {
    pub fn is_unknown(&self) -> bool {
        match self {
            Self::Unknown => true,
            _ => false,
        }
    }
}

impl<'tcx> ut::UnifyValue for TyVarValue<'tcx> {
    type Error = ut::NoError;

    fn unify_values(s: &Self, t: &Self) -> Result<Self, Self::Error> {
        match (s, t) {
            (&Self::Known(a), &Self::Known(b)) => panic!("unifying two known type variables"),
            (&Self::Known(_), _) => Ok(*s),
            (_, &Self::Known(_)) => Ok(*t),
            (&Self::Unknown, &Self::Unknown) => Ok(TyVarValue::Unknown),
        }
    }
}

/// These structs (a newtyped TyVid) are used as the unification key
/// for the `eq_relations`; they carry a `TypeVariableValue` along
/// with them.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
crate struct TyVidEqKey<'tcx> {
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

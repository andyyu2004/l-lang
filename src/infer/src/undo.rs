use crate::*;
use ena::snapshot_vec as sv;
use ena::undo_log::{Snapshots, UndoLogs};
use ena::unify as ut;
use lcore::ty;

pub enum UndoLog<'tcx> {
    TyVar(type_variables::TyVarUndoLog<'tcx>),
}

#[derive(Default)]
crate struct InferCtxUndoLogs<'tcx> {
    crate logs: Vec<UndoLog<'tcx>>,
    crate open_snapshots_count: usize,
}

impl<'tcx, T> UndoLogs<T> for InferCtxInner<'tcx>
where
    UndoLog<'tcx>: From<T>,
{
    fn num_open_snapshots(&self) -> usize {
        self.undo_logs.num_open_snapshots()
    }

    fn push(&mut self, undo: T) {
        self.undo_logs.push(undo)
    }

    fn clear(&mut self) {
        self.undo_logs.clear()
    }
}

impl<'tcx, T> UndoLogs<T> for InferCtxUndoLogs<'tcx>
where
    UndoLog<'tcx>: From<T>,
{
    fn num_open_snapshots(&self) -> usize {
        self.open_snapshots_count
    }

    fn push(&mut self, undo: T) {
        if self.in_snapshot() {
            self.logs.push(undo.into())
        }
    }

    fn clear(&mut self) {
        self.logs.clear();
        self.open_snapshots_count = 0;
    }

    fn extend<J>(&mut self, undos: J)
    where
        Self: Sized,
        J: IntoIterator<Item = T>,
    {
        if self.in_snapshot() {
            self.logs.extend(undos.into_iter().map(UndoLog::from))
        }
    }
}

macro_rules! impl_from {
    ($($ctor: ident ($ty: ty),)*) => {
        $(
        impl<'tcx> From<$ty> for UndoLog<'tcx> {
            fn from(x: $ty) -> Self {
                UndoLog::$ctor(x.into())
            }
        }
        )*
    }
}

impl_from! {
    TyVar(type_variables::TyVarUndoLog<'tcx>),
    TyVar(sv::UndoLog<ut::Delegate<type_variables::TyVidEqKey<'tcx>>>),
    TyVar(sv::UndoLog<ut::Delegate<ty::TyVid>>),
}

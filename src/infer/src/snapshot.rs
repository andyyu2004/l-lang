use crate::*;
use ena::undo_log::{Rollback, Snapshots};
use std::cell::Ref;

pub struct InferCtxSnapshot<'a, 'tcx> {
    snapshot: Snapshot<'tcx>,
    tables: Ref<'a, TypeckTables<'tcx>>,
}

impl<'tcx> InferCtxUndoLogs<'tcx> {
    crate fn start_snapshot(&mut self) -> Snapshot<'tcx> {
        self.open_snapshots_count += 1;
        Snapshot { undo_logs_count: self.logs.len(), marker: std::marker::PhantomData }
    }
}

impl<'a, 'tcx> InferCtx<'a, 'tcx> {
    crate fn rollback_to(&self, snapshot: InferCtxSnapshot<'a, 'tcx>) {
        self.inner.borrow_mut().rollback_to(snapshot.snapshot)
    }

    crate fn start_snapshot(&self) -> InferCtxSnapshot<'a, 'tcx> {
        // ensure the borrow finishes by the time we return the snapshot
        // otherwise we are sure to run into BorrowMut errors
        let snapshot = self.inner.borrow_mut().undo_logs.start_snapshot();
        InferCtxSnapshot { snapshot, tables: self.tables.borrow() }
    }
}

impl<'tcx> InferCtxInner<'tcx> {
    pub fn rollback_to(&mut self, snapshot: Snapshot<'tcx>) {
        while self.undo_logs.logs.len() > snapshot.undo_logs_count {
            let undo = self.undo_logs.logs.pop().unwrap();
            self.reverse(undo);
        }
    }
}

impl<'tcx> Rollback<UndoLog<'tcx>> for InferCtxInner<'tcx> {
    fn reverse(&mut self, undo: UndoLog<'tcx>) {
        match undo {
            UndoLog::TyVar(undo) => self.type_variable_storage.reverse(undo),
        }
    }
}

impl<'tcx> Snapshots<UndoLog<'tcx>> for InferCtxInner<'tcx> {
    type Snapshot = Snapshot<'tcx>;

    fn actions_since_snapshot(&self, snapshot: &Self::Snapshot) -> &[UndoLog<'tcx>] {
        &self.undo_logs.logs[snapshot.undo_logs_count..]
    }

    fn commit(&mut self, _snapshot: Self::Snapshot) {
        todo!()
    }

    fn start_snapshot(&mut self) -> Self::Snapshot {
        self.undo_logs.start_snapshot()
    }

    // this api is too awkward so we don't use it
    fn rollback_to<R>(&mut self, _storage: impl FnOnce() -> R, _snapshot: Self::Snapshot)
    where
        R: ena::undo_log::Rollback<UndoLog<'tcx>>,
    {
        unimplemented!()
    }
}

pub struct Snapshot<'tcx> {
    /// the number of undo logs at the point of this snapshot
    crate undo_logs_count: usize,
    marker: std::marker::PhantomData<&'tcx ()>,
}

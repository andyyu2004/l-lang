use ena::undo_log::UndoLogs;
use std::marker::PhantomData;

#[derive(Default)]
pub struct InferCtxUndoLogs<'tcx> {
    marker: PhantomData<&'tcx ()>,
}

// TODO this is unimplemented but I don't want it to crash :)
impl<'tcx, T> UndoLogs<T> for InferCtxUndoLogs<'tcx> {
    fn num_open_snapshots(&self) -> usize {
        0
    }

    fn push(&mut self, undo: T) {
    }

    fn clear(&mut self) {
    }

    fn extend<J>(&mut self, undos: J)
    where
        Self: Sized,
        J: IntoIterator<Item = T>,
    {
    }
}

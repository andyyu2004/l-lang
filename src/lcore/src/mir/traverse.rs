use crate::mir::{BlockId, Mir, ENTRY_BLOCK};
use rustc_hash::FxHashSet;

pub struct Preorder<'a, 'tcx> {
    mir: &'a Mir<'tcx>,
    visited: FxHashSet<BlockId>,
    stack: Vec<BlockId>,
}

pub fn preorder<'a, 'tcx>(mir: &'a Mir<'tcx>) -> Preorder<'a, 'tcx> {
    Preorder { mir, visited: Default::default(), stack: vec![ENTRY_BLOCK] }
}

impl<'a, 'tcx> Iterator for Preorder<'a, 'tcx> {
    type Item = BlockId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(block) = self.stack.pop() {
            if !self.visited.insert(block) {
                continue;
            }
            self.stack.extend(self.mir[block].terminator().successors());
            return Some(block);
        }
        None
    }
}

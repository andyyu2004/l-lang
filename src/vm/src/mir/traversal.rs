use super::*;
use rustc_hash::FxHashSet;

struct Preorder<'a, 'tcx> {
    body: &'a Body<'tcx>,
    stack: Vec<BlockId>,
    visited: FxHashSet<BlockId>,
}

impl<'a, 'tcx> Preorder<'a, 'tcx> {
    pub fn new(body: &'a Body<'tcx>, start: BlockId) -> Self {
        Self { body, visited: Default::default(), stack: vec![start] }
    }
}

impl<'a, 'tcx> Iterator for Preorder<'a, 'tcx> {
    type Item = (BlockId, &'a BasicBlock<'tcx>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(id) = self.stack.pop() {
            // returns true if not visited
            if !self.visited.insert(id) {
                continue;
            }
            let block = &self.body.basic_blocks[id];
            self.stack.extend(block.terminator().successors());
            return Some((id, block));
        }
        None
    }
}

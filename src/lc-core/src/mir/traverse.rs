use crate::mir::{BasicBlock, BlockId, Mir, ENTRY_BLOCK};
use itertools::Itertools;
use lc_ds::Bitset;

pub struct Preorder<'a, 'tcx> {
    mir: &'a Mir<'tcx>,
    visited: Bitset<BlockId>,
    stack: Vec<BlockId>,
}

pub fn preorder<'a, 'tcx>(mir: &'a Mir<'tcx>) -> Preorder<'a, 'tcx> {
    Preorder { mir, visited: Bitset::new(mir.len()), stack: vec![ENTRY_BLOCK] }
}

impl<'a, 'tcx> Iterator for Preorder<'a, 'tcx> {
    type Item = (BlockId, &'a BasicBlock<'tcx>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(block) = self.stack.pop() {
            if !self.visited.insert(block) {
                continue;
            }
            self.stack.extend(self.mir[block].terminator().successors());
            return Some((block, &self.mir[block]));
        }
        None
    }
}

pub struct ReversePostOrder<'a, 'tcx> {
    mir: &'a Mir<'tcx>,
    visited: Bitset<BlockId>,
    /// the blocks are in postorder
    /// but we pop from the end so the iterator is rpo
    blocks: Vec<BlockId>,
}

pub fn rpo<'a, 'tcx>(mir: &'a Mir<'tcx>) -> ReversePostOrder<'a, 'tcx> {
    let mut this =
        ReversePostOrder { mir, visited: Bitset::new(mir.len()), blocks: Default::default() };
    this.traverse(ENTRY_BLOCK);
    debug_assert_eq!(mir.len(), this.blocks.len());
    this
}

impl<'a, 'tcx> ReversePostOrder<'a, 'tcx> {
    fn traverse(&mut self, block: BlockId) {
        if !self.visited.insert(block) {
            return;
        }

        for successor in self.mir[block].terminator().successors() {
            self.traverse(successor);
        }

        // we only add the block after all its successors have been traversed
        self.blocks.push(block)
    }
}

impl<'a, 'tcx> Iterator for ReversePostOrder<'a, 'tcx> {
    type Item = (BlockId, &'a BasicBlock<'tcx>);

    // the postorder traversal is not implemented "incrementally"
    // but just calculated on construction
    fn next(&mut self) -> Option<Self::Item> {
        self.blocks.pop().map(|block| (block, &self.mir[block]))
    }
}

pub struct PostOrder<'a, 'tcx> {
    blocks: Vec<(BlockId, &'a BasicBlock<'tcx>)>,
}

pub fn postorder<'a, 'tcx>(mir: &'a Mir<'tcx>) -> PostOrder<'a, 'tcx> {
    // can't just call rev on postorder as it does not implement DoubleEndedIter
    // so we just collect it into a vector and walk it in reverse
    let reverse_postorder = rpo(mir);
    let blocks = reverse_postorder.collect_vec();
    PostOrder { blocks }
}

impl<'a, 'tcx> Iterator for PostOrder<'a, 'tcx> {
    type Item = (BlockId, &'a BasicBlock<'tcx>);

    fn next(&mut self) -> Option<Self::Item> {
        self.blocks.pop()
    }
}

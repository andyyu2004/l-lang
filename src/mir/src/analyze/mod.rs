use crate::traverse;
use index::Idx;
use itertools::Itertools;
use lcore::mir::{BlockId, Mir};
use lcore::TyCtx;
use rustc_hash::FxHashSet;

pub fn analyze<'a, 'tcx>(_tcx: TyCtx<'tcx>, mir: &'a mut Mir<'tcx>) {
    self::remove_dead_blocks(mir);
}

#[cfg(test)]
mod test {
    use super::*;
    use lcore::mir::BasicBlock;

    #[test]
    fn test_removal_dead_blocks() {
        let mut mir = Mir::default();
        mir.basic_blocks = vec![BasicBlock::default(); 4].into_iter().collect();
    }
}

/// remove's unreachable blocks
/// this should be run before `typecheck` as some unreachable blocks
/// may be type incorrect
fn remove_dead_blocks<'a, 'tcx>(mir: &'a mut Mir<'tcx>) {
    let reachable = traverse::preorder(mir).collect::<FxHashSet<_>>();
    let mut reachable = reachable.into_iter().map(|idx| idx.index()).collect_vec();
    reachable.sort();

    // this number is to essentially ensure an error if it isn't overwritten
    let mut swaps = (0..mir.len()).map(BlockId::new).collect_vec();

    // reorder all the blocks such that all the dead blocks are at the end
    // this algorithm does not preserve order in any sense
    //
    // example
    //
    // consider the following example
    // mir.basic_blocks = [bb0,bb1,bb2,bb3]
    // reachable = { 1, 3 }
    //
    // first iteration (i = 0; block = 1)
    // swaps[1] = 0 (so we move bb1 to bb0 as bb0 is unreachable)
    // mir.basic_blocks = [bb1,bb0,bb2,bb3] (after swap)
    //
    // second iteration (i = 1; block = 3)
    // swaps[3] = 1
    // mir.basic_blocks = [bb1,bb3,bb2,bb0] (after swap)
    //
    // swaps = [0,0,2,1]
    //
    // truncate (i = 2)
    // mir.basic_blocks = [bb1, bb3]
    //
    //
    // *update successors*
    //
    // consider the following example
    // mir.basic_blocks = [bb0,bb1,bb2,bb3]
    // reachable = { 3, 1 }
    // swaps[3] = 0;
    // [bb3,bb1,bb2,bb0]
    //
    // swaps[1] = 1;
    //
    // bbs = [bb3,bb1]
    // swaps = [0, 1, 2, 0]
    let mut i = 0;
    for &block in &reachable {
        swaps[block] = BlockId::new(i);
        if i != block {
            mir.raw.swap(block, i);
        }
        i += 1
    }

    debug_assert_eq!(i, reachable.len());
    mir.truncate(i);

    for block in &mut mir.basic_blocks {
        for successor in block.terminator_mut().successors_mut() {
            debug_assert!(reachable.contains(&successor.index()));
            *successor = swaps[successor.index()]
        }
    }
}

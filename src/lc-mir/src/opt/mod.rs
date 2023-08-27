use itertools::Itertools;
use lc_core::mir::{self, BlockId, Mir};
use lc_core::TyCtx;
use lc_ds::Bitset;
use lc_index::Idx;

pub fn early_opt<'tcx>(_tcx: TyCtx<'tcx>, mir: &mut Mir<'tcx>) {
    self::remove_dead_blocks(mir);
}

pub fn late_opt<'tcx>(_tcx: TyCtx<'tcx>, _mir: &mut Mir<'tcx>) {
}

/// remove's unreachable blocks
/// this should be run before `typecheck` as some unreachable blocks
/// may be type incorrect
fn remove_dead_blocks(mir: &mut Mir<'_>) {
    let mut reachable = Bitset::new(mir.len());
    mir::preorder(mir).for_each(|(block_id, _)| reachable.set(block_id));

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
    // *update successors*

    let mut i = 0;
    // note that this iteration will occur in ascending order of block ids
    // due to how bitsets work
    // in a prior implementation using HashSet (i.e. unordered iteration)
    // the algorithm did not work (unsure why this is the case)
    for block in &reachable {
        let block = block.index();
        swaps[block] = BlockId::new(i);
        if i != block {
            mir.raw.swap(block, i);
        }
        i += 1
    }

    mir.truncate(i);

    for block in &mut mir.basic_blocks {
        for successor in block.terminator_mut().successors_mut() {
            *successor = swaps[successor.index()]
        }
    }
}

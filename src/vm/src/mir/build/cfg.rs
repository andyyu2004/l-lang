//! control flow graph

use super::{BasicBlock, BlockId};
use crate::mir;
use indexed_vec::IndexVec;
use mir::{Lvalue, Rvalue, SpanInfo, Terminator, TerminatorKind};

#[derive(Default)]
pub struct Cfg<'tcx> {
    pub(super) basic_blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
}

impl<'tcx> Cfg<'tcx> {
    pub fn append_basic_block(&mut self) -> BlockId {
        self.basic_blocks.push(BasicBlock::default())
    }

    fn block_mut(&mut self, block: BlockId) -> &mut BasicBlock<'tcx> {
        &mut self.basic_blocks[block]
    }

    /// branch inst
    pub fn br(&mut self, info: SpanInfo, from: BlockId, to: BlockId) {
        self.terminate(info, from, TerminatorKind::Branch(to))
    }

    pub fn terminate(&mut self, info: SpanInfo, block: BlockId, kind: TerminatorKind<'tcx>) {
        let block = self.block_mut(block);
        debug_assert!(block.terminator.is_none(), "block already has terminator");
        block.terminator = Some(Terminator { info, kind })
    }

    /// push a statement onto the given block
    pub fn push(&mut self, block: BlockId, stmt: mir::Stmt<'tcx>) {
        self.basic_blocks[block].stmts.push(stmt);
    }

    pub fn push_unit(&mut self, info: SpanInfo, block: BlockId, lvalue: Lvalue<'tcx>) {
        todo!();
        // self.push_assignment(info, block, lvalue, Rvalue::Ref());
    }

    pub fn push_assignment(
        &mut self,
        info: SpanInfo,
        block: BlockId,
        lvalue: Lvalue<'tcx>,
        rvalue: Rvalue<'tcx>,
    ) {
        self.push(block, mir::Stmt { info, kind: mir::StmtKind::Assign(box (lvalue, rvalue)) });
    }
}

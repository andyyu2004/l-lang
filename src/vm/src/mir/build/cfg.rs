//! control flow graph

use super::{BasicBlock, BlockId, Builder};
use crate::mir;
use crate::ty::Const;
use crate::typeck::TyCtx;
use indexed_vec::IndexVec;
use mir::{Lvalue, Operand, Rvalue, SpanInfo, Terminator, TerminatorKind};

#[derive(Default)]
pub struct Cfg<'tcx> {
    pub(super) basic_blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
}

impl<'tcx> Cfg<'tcx> {
    pub fn append_basic_block(&mut self) -> BlockId {
        self.basic_blocks.push(BasicBlock::default())
    }
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn append_basic_block(&mut self) -> BlockId {
        self.cfg.append_basic_block()
    }

    fn block_mut(&mut self, block: BlockId) -> &mut BasicBlock<'tcx> {
        &mut self.cfg.basic_blocks[block]
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
        self.cfg.basic_blocks[block].stmts.push(stmt);
    }

    /// writes a unit into `lvalue`
    pub fn push_assign_unit(&mut self, info: SpanInfo, block: BlockId, lvalue: Lvalue<'tcx>) {
        let unit = self.tcx.intern_const(Const::unit());
        self.push_assignment(info, block, lvalue, Rvalue::Use(Operand::Const(unit)));
    }

    pub fn push_assignment(
        &mut self,
        info: SpanInfo,
        block: BlockId,
        lvalue: Lvalue<'tcx>,
        rvalue: Rvalue<'tcx>,
    ) {
        self.push(block, mir::Stmt { info, kind: mir::StmtKind::Assign(lvalue, rvalue) });
    }
}

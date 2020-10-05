//! control flow graph

use super::{BasicBlock, BlockId, Builder, ReleaseInfo, VarId, ENTRY_BLOCK};
use crate::mir;
use crate::ty::{Const, TyKind};
use crate::typeck::TyCtx;
use indexed_vec::{Idx, IndexVec};
use mir::{Lvalue, Operand, Rvalue, SpanInfo, Terminator, TerminatorKind};

pub struct Cfg<'tcx> {
    pub(super) basic_blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
}

impl Default for Cfg<'_> {
    fn default() -> Self {
        let mut cfg = Self { basic_blocks: Default::default() };
        assert_eq!(cfg.append_basic_block(), ENTRY_BLOCK);
        cfg
    }
}

impl<'tcx> Cfg<'tcx> {
    pub fn append_basic_block(&mut self) -> BlockId {
        self.basic_blocks.push(BasicBlock::default())
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

    pub fn push(&mut self, block: BlockId, stmt: mir::Stmt<'tcx>) {
        self.basic_blocks[block].stmts.push(stmt);
    }

    fn block_mut(&mut self, block: BlockId) -> &mut BasicBlock<'tcx> {
        &mut self.basic_blocks[block]
    }

    pub fn terminate(&mut self, info: SpanInfo, block: BlockId, kind: TerminatorKind<'tcx>) {
        let block = self.block_mut(block);
        debug_assert!(block.terminator.is_none(), "block already has terminator");
        block.terminator = Some(Terminator { info, kind })
    }
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn append_basic_block(&mut self) -> BlockId {
        self.cfg.append_basic_block()
    }

    fn block_mut(&mut self, block: BlockId) -> &mut BasicBlock<'tcx> {
        self.cfg.block_mut(block)
    }

    /// branch inst
    pub fn branch(&mut self, info: SpanInfo, from: BlockId, to: BlockId) {
        self.terminate(info, from, TerminatorKind::Branch(to))
    }

    pub fn terminate(&mut self, info: SpanInfo, block: BlockId, kind: TerminatorKind<'tcx>) {
        self.cfg.terminate(info, block, kind)
    }

    pub fn push_release(&mut self, block: BlockId, release: ReleaseInfo<'tcx>) {
        let ReleaseInfo { info, lvalue } = release;
        self.push(block, mir::Stmt { info, kind: mir::StmtKind::Release(lvalue) })
    }

    pub fn push_retain(&mut self, info: SpanInfo, block: BlockId, lvalue: Lvalue<'tcx>) {
        self.push(block, mir::Stmt { info, kind: mir::StmtKind::Retain(lvalue) })
    }

    /// push a statement onto the given block
    pub fn push(&mut self, block: BlockId, stmt: mir::Stmt<'tcx>) {
        self.cfg.basic_blocks[block].stmts.push(stmt);
    }

    /// writes a unit into `lvalue`
    pub fn push_assign_unit(&mut self, info: SpanInfo, block: BlockId, lvalue: Lvalue<'tcx>) {
        let unit = self.tcx.intern_const(Const::unit(self.tcx));
        self.push_assignment(info, block, lvalue, Rvalue::Operand(Operand::Const(unit)));
    }

    pub fn push_assignment(
        &mut self,
        info: SpanInfo,
        block: BlockId,
        lvalue: Lvalue<'tcx>,
        rvalue: Rvalue<'tcx>,
    ) {
        self.cfg.push_assignment(info, block, lvalue, rvalue);
        // if the type is pointer, then it is a box and we need to do refcounting
        if lvalue.ty(self).is_ptr() {
            self.push_retain(info, block, lvalue);
            self.schedule_release(info, lvalue);
        }
    }
}

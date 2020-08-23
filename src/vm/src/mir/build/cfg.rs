//! control flow graph

use super::{BasicBlock, BlockId};
use crate::mir;
use indexed_vec::IndexVec;
use mir::{Lvalue, Rvalue, SpanInfo};

#[derive(Default)]
pub struct Cfg<'tcx> {
    pub(super) basic_blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
}

impl<'tcx> Cfg<'tcx> {
    pub fn push(&mut self, block: BlockId, stmt: mir::Stmt<'tcx>) {
        self.basic_blocks[block].stmts.push(stmt);
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

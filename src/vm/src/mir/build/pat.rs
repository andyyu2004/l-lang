use super::{BlockAnd, BlockAndExt, Builder};
use crate::mir::{BlockId, Lvalue, Rvalue, VarId, VarKind};
use crate::set;
use crate::tir;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn declare_pat(&mut self, block: BlockId, pat: &tir::Pattern<'tcx>) -> BlockAnd<VarId> {
        let info = self.span_info(pat.span);
        match pat.kind {
            tir::PatternKind::Wildcard => todo!(),
            tir::PatternKind::Binding(ident, _) => block.and(self.alloc_local(pat)),
            tir::PatternKind::Field(_) => todo!(),
            tir::PatternKind::Lit(_) => unreachable!(),
        }
    }
}

use crate::{set, BlockAnd, BlockAndExt, MirBuilder};
use lcore::mir::*;

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
    /// binds each subpattern to the relevant value
    ///
    /// let l = (3,4);
    /// bind (a, b) l
    /// a <- l.0; b <- l.1
    ///
    crate fn bind_pat_to_lvalue(
        &mut self,
        mut block: BlockId,
        irref_pat: &tir::Pattern<'tcx>,
        lvalue: Lvalue<'tcx>,
    ) -> BlockAnd<()> {
        debug_assert!(irref_pat.is_irrefutable());
        let info = self.span_info(irref_pat.span);
        match irref_pat.kind {
            // we know `pat` is also irrefutable as the outer pat was irrefutable
            tir::PatternKind::Box(ref pat) =>
                self.bind_pat_to_lvalue(block, pat, self.tcx.project_deref(lvalue)),
            tir::PatternKind::Binding(m, _, _) => {
                let &tir::Pattern { id, span, ty, .. } = irref_pat;
                let rvalue = Rvalue::Operand(Operand::Lvalue(lvalue));
                let local = self.alloc_local(id, span, ty);
                self.vars[local].mtbl = m;
                self.push_assignment(info, block, local.into(), rvalue);
                block.unit()
            }
            tir::PatternKind::Field(ref fs) => {
                // field patterns are implemented by creating projections
                // let pair = (1,2);
                // let (x, y) = pair;
                // implemented as
                // pair <- (1,2);
                // x    <- pair.0;
                // y    <- pair.1;
                //
                for f in fs {
                    let lvalue = self.tcx.project_field(lvalue, f.index, f.pat.ty);
                    set!(block = self.bind_pat_to_lvalue(block, &f.pat, lvalue));
                }
                block.unit()
            }
            tir::PatternKind::Lit(_) => panic!("refutable binding"),
            tir::PatternKind::Variant(..) => todo!(),
            tir::PatternKind::Wildcard => block.unit(),
        }
    }
}

use super::{BlockAnd, BlockAndExt, Builder, LvalueBuilder};
use crate::mir::{BlockId, Lvalue, Operand, Rvalue, VarId, VarKind};
use crate::set;
use crate::tir;
use crate::ty::Projection;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    crate fn bind_pat_to_lvalue(
        &mut self,
        mut block: BlockId,
        irref_pat: &tir::Pattern<'tcx>,
        lvalue: Lvalue<'tcx>,
    ) -> BlockAnd<()> {
        debug_assert!(!irref_pat.is_refutable());
        let info = self.span_info(irref_pat.span);
        match irref_pat.kind {
            tir::PatternKind::Wildcard => block.unit(),
            tir::PatternKind::Binding(_, _) => {
                let rvalue = Rvalue::Use(Operand::Ref(lvalue));
                let lvalue = self.alloc_local(irref_pat).into();
                self.push_assignment(info, block, lvalue, rvalue);
                block.unit()
            }
            tir::PatternKind::Field(fs) => {
                // field patterns are implemented by creating projections
                // let pair = (1,2);
                // let (x, y) = pair;
                // implemented as
                // x <- pair.0;
                // y <- pair.1;
                for f in fs {
                    let lvalue = self.tcx.lvalue_project_field(lvalue, f.field, f.pat.ty);
                    set!(block = self.bind_pat_to_lvalue(block, f.pat, lvalue));
                }
                block.unit()
            }
            tir::PatternKind::Lit(_) => panic!("refutable binding"),
        }
    }
}

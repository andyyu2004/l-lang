use crate::Visitor;
use lcore::mir::Operand;
use lcore::ty::TyCtx;

struct MonoCollector<'tcx> {
    tcx: TyCtx<'tcx>,
}

impl<'tcx> Visitor<'tcx> for MonoCollector<'tcx> {
    fn visit_operand(&mut self, operand: &Operand<'tcx>) {
        if let Operand::Instance(instance) = operand {
            todo!()
        }
    }
}

use crate::mir::{self, *};
use crate::typeck::inference::InferCtx;
use crate::typeck::TyCtx;

// this just panics on typecheck failure as this is more of an internal compiler check
// rather than a user check
struct Typechecker<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    infcx: &'a InferCtx<'a, 'tcx>,
    mir: &'a mir::Mir<'tcx>,
}

impl<'tcx> Visitor<'tcx> for Typechecker<'_, 'tcx> {
    fn visit_assignment(&mut self, lvalue: &Lvalue<'tcx>, rvalue: &Rvalue<'tcx>) {
        let lvalue_ty = self.check_lvalue(lvalue);
        let rvalue_ty = self.check_rvalue(rvalue);
        assert_eq!(lvalue_ty, rvalue_ty);
    }
}

impl<'a, 'tcx> Typechecker<'a, 'tcx> {
    pub fn typecheck(&mut self) {
        self.visit_mir(&self.mir);
    }

    fn check_lvalue(&mut self, lvalue: &Lvalue<'tcx>) -> Ty<'tcx> {
        let base_ty = self.mir.vars[lvalue.id].ty;
        lvalue.projs.iter().fold(base_ty, |ty, proj| self.tcx.apply_projection(ty, proj))
    }

    fn check_rvalue(&mut self, rvalue: &Rvalue<'tcx>) -> Ty<'tcx> {
        match rvalue {
            Rvalue::Operand(operand) => self.check_operand(operand),
            Rvalue::Unary(_, _) => todo!(),
            Rvalue::Bin(_, _, _) => todo!(),
            Rvalue::Box(_) => todo!(),
            Rvalue::Ref(_) => todo!(),
            Rvalue::Closure(_, _) => todo!(),
            Rvalue::Adt { adt, variant_idx, substs, fields } => todo!(),
            Rvalue::Discriminant(_) => todo!(),
        }
    }

    fn check_operand(&mut self, operand: &Operand<'tcx>) -> Ty<'tcx> {
        match operand {
            Operand::Lvalue(lvalue) => self.check_lvalue(lvalue),
            Operand::Const(c) => c.ty,
            Operand::Item(def) => self.tcx.collected_ty(*def),
        }
    }
}

pub fn check<'a, 'tcx>(mir: &'a mir::Mir<'tcx>, infcx: &InferCtx<'a, 'tcx>) {
    Typechecker { infcx, mir, tcx: infcx.tcx }.typecheck();
}

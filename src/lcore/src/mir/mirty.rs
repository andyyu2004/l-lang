use crate::mir::*;
use crate::ty::Subst;

pub trait MirTy<'tcx> {
    fn ty(&self, tcx: TyCtx<'tcx>, vars: &impl LvalueTy<'tcx>) -> Ty<'tcx>;
}

impl<'tcx> MirTy<'tcx> for Operand<'tcx> {
    fn ty(&self, tcx: TyCtx<'tcx>, vars: &impl LvalueTy<'tcx>) -> Ty<'tcx> {
        match self {
            Operand::Lvalue(lvalue) => lvalue.ty(tcx, vars),
            Operand::Item(def_id, substs) => tcx.type_of(*def_id).subst(tcx, substs),
            Operand::Const(c) => c.ty,
        }
    }
}

impl<'tcx> MirTy<'tcx> for Lvalue<'tcx> {
    fn ty(&self, tcx: TyCtx<'tcx>, vars: &impl LvalueTy<'tcx>) -> Ty<'tcx> {
        let base = vars.locals()[self.id].ty;
        self.projs.iter().fold(base, |ty, proj| tcx.apply_projection(ty, proj))
    }
}

pub trait LvalueTy<'tcx> {
    fn locals(&self) -> &IndexVec<VarId, Var<'tcx>>;
}

impl<'tcx> LvalueTy<'tcx> for Mir<'tcx> {
    fn locals(&self) -> &IndexVec<VarId, Var<'tcx>> {
        &self.vars
    }
}

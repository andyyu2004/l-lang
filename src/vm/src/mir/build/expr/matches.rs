use super::Builder;
use crate::mir::*;
use crate::tir;
use crate::ty;
use ty::TyKind;

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn lower_match(
        &mut self,
        mut block: BlockId,
        dest: Lvalue<'tcx>,
        expr: &tir::Expr<'tcx>,
        scrut: &tir::Expr<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) {
        todo!()
    }
}

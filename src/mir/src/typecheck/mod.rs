//! this module performs sanity type checks on mir
//! there are not intended to be any errors so it panics on error

use crate::MirVisitor;
use lcore::mir::*;
use lcore::ty::{Ty, TyCtx};

pub fn typecheck<'tcx>(tcx: TyCtx<'tcx>, mir: &'tcx Mir<'tcx>) {
    Typechecker::new(tcx, mir).typecheck()
}

struct Typechecker<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    mir: &'a Mir<'tcx>,
}

impl<'a, 'tcx> Typechecker<'a, 'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, mir: &'tcx Mir<'tcx>) -> Self {
        Self { tcx, mir }
    }

    fn typecheck(&mut self) {
        self.visit_mir(self.mir);
    }

    fn lvalue_ty(&mut self, lvalue: &Lvalue<'tcx>) -> Ty<'tcx> {
        lvalue.ty(self.tcx, self.mir)
    }

    fn op_ty(&mut self, operand: &Operand<'tcx>) -> Ty<'tcx> {
        operand.ty(self.tcx, self.mir)
    }

    fn rvalue_ty(&mut self, rvalue: &Rvalue<'tcx>) -> Ty<'tcx> {
        let tcx = self.tcx;
        match rvalue {
            // the unary operators (!,-) at mir level do not change the operands type
            Rvalue::Unary(_, operand) | Rvalue::Operand(operand) => self.op_ty(operand),
            Rvalue::Box(operand) => tcx.mk_box_ty(self.op_ty(operand)),
            Rvalue::Ref(lvalue) => tcx.mk_ptr_ty(self.lvalue_ty(lvalue)),
            Rvalue::Discriminant(_) => tcx.types.discr,
            Rvalue::Closure(_, _) => todo!(),
            Rvalue::Bin(op, l, r) => {
                let lty = self.op_ty(l);
                let rty = self.op_ty(r);
                assert_eq!(lty, rty);
                match op {
                    ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => lty,
                    ast::BinOp::Lt | ast::BinOp::Gt | ast::BinOp::Eq | ast::BinOp::Neq =>
                        tcx.types.bool,
                    ast::BinOp::And | ast::BinOp::Or => {
                        assert_eq!(lty, tcx.types.bool);
                        tcx.types.bool
                    }
                }
            }
            Rvalue::Adt { adt, variant_idx, substs, fields } => {
                adt.variants[*variant_idx]
                    .fields
                    .iter()
                    .map(|field| field.ty(tcx, substs))
                    .zip(fields)
                    .for_each(|(ty, op)| assert_eq!(ty, self.op_ty(op)));
                tcx.mk_adt_ty(adt, substs)
            }
        }
    }
}

impl<'a, 'tcx> MirVisitor<'tcx> for Typechecker<'a, 'tcx> {
    fn visit_assignment(&mut self, lvalue: &Lvalue<'tcx>, rvalue: &Rvalue<'tcx>) {
        let lvalue_ty = self.lvalue_ty(lvalue);
        let rvalue_ty = self.rvalue_ty(rvalue);
        assert_eq!(lvalue_ty, rvalue_ty);
    }
}

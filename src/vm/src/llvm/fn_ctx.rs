use super::CodegenCtx;
use crate::ast;
use crate::mir::{self, VarId};
use crate::ty::ConstKind;
use indexed_vec::Idx;
use inkwell::{values::*, FloatPredicate};
use rustc_hash::FxHashMap;
use std::ops::Deref;

pub(super) struct FnCtx<'a, 'tcx> {
    cctx: &'a CodegenCtx<'tcx>,
    body: &'tcx mir::Body<'tcx>,
    function: FunctionValue<'tcx>,
    vars: FxHashMap<mir::VarId, Var<'tcx>>,
}

#[derive(Debug, Clone, Copy)]
struct Var<'tcx> {
    ptr: PointerValue<'tcx>,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn new(
        cctx: &'a CodegenCtx<'tcx>,
        body: &'tcx mir::Body<'tcx>,
        function: FunctionValue<'tcx>,
    ) -> Self {
        Self { cctx, body, function, vars: Default::default() }
    }

    crate fn codegen_body(&mut self, body: &'tcx mir::Body<'tcx>) {
        for (id, &var) in body.vars.iter_enumerated() {
            self.alloca(id, var);
        }
        for basic_block in &body.basic_blocks {
            self.compile_basic_block(basic_block);
        }
    }

    fn alloca(&mut self, id: VarId, var: mir::Var<'tcx>) -> Var<'tcx> {
        let ptr = self.build_alloca(self.llvm_ty(var.ty), &var.to_string());
        let var = Var { ptr };
        self.vars.insert(id, var);
        var
    }

    fn compile_basic_block(&mut self, basic_block: &mir::BasicBlock) {
        basic_block.stmts.iter().for_each(|stmt| self.compile_stmt(stmt));
        self.codegen_terminator(basic_block.terminator());
    }

    fn compile_stmt(&mut self, stmt: &mir::Stmt) {
        match &stmt.kind {
            mir::StmtKind::Assign(lvalue, rvalue) => {
                let val = self.codegen_rvalue(rvalue);
                let var = self.vars[&lvalue.var];
                self.build_store(var.ptr, val);
            }
            mir::StmtKind::Nop => {}
        }
    }

    fn codegen_operand(&mut self, operand: &mir::Operand) -> BasicValueEnum<'tcx> {
        match operand {
            mir::Operand::Const(c) => match c.kind {
                ConstKind::Floating(f) => self.llctx.f64_type().const_float(f).into(),
                ConstKind::Bool(b) => self.llctx.bool_type().const_int(b, true).into(),
            },
            mir::Operand::Ref(lvalue) => {
                let var = self.vars[&lvalue.var];
                self.build_load(var.ptr, "load").into()
            }
        }
    }

    fn codegen_rvalue(&mut self, rvalue: &mir::Rvalue) -> BasicValueEnum<'tcx> {
        match rvalue {
            mir::Rvalue::Use(operand) => self.codegen_operand(operand),
            mir::Rvalue::Bin(op, l, r) => {
                let lhs = self.codegen_operand(l).into_float_value();
                let rhs = self.codegen_operand(r).into_float_value();
                match op {
                    ast::BinOp::Mul => self.build_float_mul(lhs, rhs, "tmpfmul").into(),
                    ast::BinOp::Div => self.build_float_div(lhs, rhs, "tmpfdiv").into(),
                    ast::BinOp::Add => self.build_float_add(lhs, rhs, "tmpadd").into(),
                    ast::BinOp::Sub => self.build_float_sub(lhs, rhs, "tmpfsub").into(),
                    ast::BinOp::Lt | ast::BinOp::Gt => self.compile_cmp(*op, lhs, rhs).into(),
                }
            }
        }
    }

    fn compile_cmp(
        &mut self,
        op: ast::BinOp,
        l: FloatValue<'tcx>,
        r: FloatValue<'tcx>,
    ) -> IntValue<'tcx> {
        match op {
            ast::BinOp::Lt =>
                self.builder.build_float_compare(FloatPredicate::OLT, l, r, "fcmp_lt"),
            ast::BinOp::Gt =>
                self.builder.build_float_compare(FloatPredicate::OGT, l, r, "fcmp_gt"),
            ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => unreachable!(),
        }
    }

    fn codegen_terminator(&mut self, terminator: &mir::Terminator) {
        match &terminator.kind {
            mir::TerminatorKind::Return => {
                let var = self.vars[&VarId::new(mir::RETURN)];
                let val = self.build_load(var.ptr, "load_ret");
                let dyn_val = &val as &dyn BasicValue;
                self.build_return(Some(dyn_val));
            }
            mir::TerminatorKind::Unreachable => {
                self.builder.build_unreachable();
            }
            mir::TerminatorKind::Branch(_) => todo!(),
            mir::TerminatorKind::Call { f, args } => todo!(),
            mir::TerminatorKind::Switch { discr, arms, default } => todo!(),
        }
    }
}

impl<'a, 'tcx> Deref for FnCtx<'a, 'tcx> {
    type Target = CodegenCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.cctx
    }
}

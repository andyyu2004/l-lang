use super::CodegenCtx;
use crate::ast;
use crate::mir::{self, BlockId, VarId};
use crate::ty::ConstKind;
use indexed_vec::{Idx, IndexVec};
use inkwell::{basic_block::BasicBlock, values::*, FloatPredicate, IntPredicate};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::ops::Deref;

pub(super) struct FnCtx<'a, 'tcx> {
    cctx: &'a CodegenCtx<'tcx>,
    body: &'tcx mir::Body<'tcx>,
    llfn: FunctionValue<'tcx>,
    vars: IndexVec<mir::VarId, Var<'tcx>>,
    /// map from mir block to llvm block
    blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
}

// TODO
struct BlockCtx {}

#[derive(Debug, Clone, Copy)]
struct Var<'tcx> {
    ptr: PointerValue<'tcx>,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn new(
        cctx: &'a CodegenCtx<'tcx>,
        body: &'tcx mir::Body<'tcx>,
        llfn: FunctionValue<'tcx>,
    ) -> Self {
        let blocks = body
            .basic_blocks
            .indices()
            .map(|i| cctx.llctx.append_basic_block(llfn, &format!("basic_block{:?}", i)))
            .collect();
        let mut ctx = Self { cctx, body, llfn, vars: Default::default(), blocks };
        ctx.set_block(BlockId::new(0));
        ctx.vars = ctx.alloc_vars();
        ctx
    }

    fn alloc_vars(&mut self) -> IndexVec<VarId, Var<'tcx>> {
        let alloca = |var_id| {
            let mir_var = self.body.vars[var_id];
            let ptr = self.build_alloca(self.llvm_ty(mir_var.ty), &mir_var.to_string());
            Var { ptr }
        };

        // store arguments into the respective vars
        let args = self.body.arg_iter().zip(self.llfn.get_param_iter()).map(|(id, llval)| {
            let var = alloca(id);
            // store the provided argument into the local variable we provided for args
            self.build_store(var.ptr, llval);
            var
        });

        let retvar = alloca(VarId::new(mir::RETURN));
        let vars = self.body.var_iter().map(alloca);
        std::iter::once(retvar).chain(args).chain(vars).collect()
    }

    crate fn codegen_body(&mut self) {
        for id in self.body.basic_blocks.indices() {
            self.codegen_basic_block(id);
        }
    }

    /// sets the current llvm block to write to
    fn set_block(&self, block: BlockId) -> &'tcx mir::BasicBlock<'tcx> {
        self.position_at_end(self.blocks[block]);
        &self.body.basic_blocks[block]
    }

    fn codegen_basic_block(&mut self, id: BlockId) -> BasicBlock<'tcx> {
        let block = self.set_block(id);
        block.stmts.iter().for_each(|stmt| self.codegen_stmt(stmt));
        self.codegen_terminator(block.terminator());
        self.blocks[id]
    }

    fn codegen_stmt(&mut self, stmt: &mir::Stmt) {
        match &stmt.kind {
            mir::StmtKind::Assign(lvalue, rvalue) => {
                let val = self.codegen_rvalue(rvalue);
                let var = self.vars[lvalue.id];
                self.build_store(var.ptr, val);
            }
            mir::StmtKind::Nop => {}
        }
    }

    fn codegen_operand(&mut self, operand: &mir::Operand) -> BasicValueEnum<'tcx> {
        match operand {
            mir::Operand::Const(c) => match c.kind {
                ConstKind::Float(f) => self.types.float.const_float(f).into(),
                ConstKind::Int(i) => self.types.int.const_int(i as u64, true).into(),
                ConstKind::Bool(b) => self.types.boolean.const_int(b as u64, true).into(),
                ConstKind::Unit => self.vals.unit.into(),
            },
            mir::Operand::Ref(lvalue) => {
                let var = self.vars[lvalue.id];
                self.build_load(var.ptr, "load").into()
            }
            mir::Operand::Item(def_id) => {
                // TODO assume item is fn for now
                let ident = self.items.borrow()[def_id];
                let llfn = self.module.get_function(ident.as_str()).unwrap();
                // probably not the `correct` way to do this :)
                unsafe { std::mem::transmute::<FunctionValue, PointerValue>(llfn) }.into()
            }
        }
    }

    fn codegen_rvalue(&mut self, rvalue: &mir::Rvalue) -> BasicValueEnum<'tcx> {
        match rvalue {
            mir::Rvalue::Use(operand) => self.codegen_operand(operand),
            mir::Rvalue::Bin(op, l, r) => {
                let lhs = self.codegen_operand(l);
                let rhs = self.codegen_operand(r);
                match (lhs, rhs) {
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) =>
                        self.codegen_float_op(*op, l, r),
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) =>
                        self.codegen_int_op(*op, l, r),
                    _ => unreachable!(),
                }
            }
            mir::Rvalue::Tuple(xs) => {
                let operands = xs.iter().map(|x| self.codegen_operand(x)).collect_vec();
                self.llctx.const_struct(&operands, false).into()
            }
        }
    }

    fn codegen_int_op(
        &mut self,
        op: ast::BinOp,
        lhs: IntValue<'tcx>,
        rhs: IntValue<'tcx>,
    ) -> BasicValueEnum<'tcx> {
        match op {
            ast::BinOp::Mul => self.build_int_mul(lhs, rhs, "tmpimul").into(),
            ast::BinOp::Div => self.build_int_signed_div(lhs, rhs, "tmpidiv").into(),
            ast::BinOp::Add => self.build_int_add(lhs, rhs, "tmpidd").into(),
            ast::BinOp::Sub => self.build_int_sub(lhs, rhs, "tmpisub").into(),
            ast::BinOp::Lt | ast::BinOp::Gt => self.compile_icmp(op, lhs, rhs).into(),
        }
    }

    fn codegen_float_op(
        &mut self,
        op: ast::BinOp,
        lhs: FloatValue<'tcx>,
        rhs: FloatValue<'tcx>,
    ) -> BasicValueEnum<'tcx> {
        match op {
            ast::BinOp::Mul => self.build_float_mul(lhs, rhs, "tmpfmul").into(),
            ast::BinOp::Div => self.build_float_div(lhs, rhs, "tmpfdiv").into(),
            ast::BinOp::Add => self.build_float_add(lhs, rhs, "tmpadd").into(),
            ast::BinOp::Sub => self.build_float_sub(lhs, rhs, "tmpfsub").into(),
            ast::BinOp::Lt | ast::BinOp::Gt => self.compile_fcmp(op, lhs, rhs).into(),
        }
    }

    fn compile_icmp(
        &mut self,
        op: ast::BinOp,
        l: IntValue<'tcx>,
        r: IntValue<'tcx>,
    ) -> IntValue<'tcx> {
        match op {
            ast::BinOp::Lt => self.builder.build_int_compare(IntPredicate::SLT, l, r, "icmp_lt"),
            ast::BinOp::Gt => self.builder.build_int_compare(IntPredicate::SGT, l, r, "icmp_gt"),
            ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => unreachable!(),
        }
    }

    fn compile_fcmp(
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
                let var = self.vars[VarId::new(mir::RETURN)];
                let val = self.build_load(var.ptr, "load_ret");
                let dyn_val = &val as &dyn BasicValue;
                self.build_return(Some(dyn_val));
            }
            mir::TerminatorKind::Unreachable => {
                self.builder.build_unreachable();
            }
            mir::TerminatorKind::Branch(block) => {
                self.build_unconditional_branch(self.blocks[*block]);
            }
            mir::TerminatorKind::Call { f, args, lvalue, target, unwind } => {
                let f = self.codegen_operand(f).into_pointer_value();
                let args = args.iter().map(|arg| self.codegen_operand(arg)).collect_vec();
                let value = self.build_call(f, &args, "fcall").try_as_basic_value().left().unwrap();
                let var = self.vars[lvalue.id];
                self.build_store(var.ptr, value);
                self.build_unconditional_branch(self.blocks[*target]);
            }
            mir::TerminatorKind::Switch { discr, arms, default } =>
                self.codegen_switch(discr, arms, *default),
        }
    }

    fn codegen_switch(
        &mut self,
        discr: &mir::Rvalue,
        arms: &[(mir::Rvalue, BlockId)],
        default: BlockId,
    ) {
        let discr = self.codegen_rvalue(discr).into_int_value();
        let arms = arms
            .iter()
            .map(|&(ref rvalue, block)| {
                let rvalue = self.codegen_rvalue(rvalue).into_int_value();
                let block = self.blocks[block];
                (rvalue, block)
            })
            .collect_vec();
        self.build_switch(discr, self.blocks[default], &arms);
    }
}

impl<'a, 'tcx> Deref for FnCtx<'a, 'tcx> {
    type Target = CodegenCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.cctx
    }
}

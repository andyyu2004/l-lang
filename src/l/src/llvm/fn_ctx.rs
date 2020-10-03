use super::util::LLVMAsPtrVal;
use super::CodegenCtx;
use crate::ast::BinOp;
use crate::mir::{self, BlockId, VarId};
use crate::ty::{AdtKind, ConstKind, Projection, Ty};
use indexed_vec::{Idx, IndexVec};
use inkwell::basic_block::BasicBlock;
use inkwell::values::*;
use inkwell::{AddressSpace, AtomicOrdering, AtomicRMWBinOp, FloatPredicate, IntPredicate};
use itertools::Itertools;
use mir::Lvalue;
use rustc_hash::FxHashMap;
use std::ops::Deref;

pub struct FnCtx<'a, 'tcx> {
    cctx: &'a CodegenCtx<'tcx>,
    body: &'tcx mir::Mir<'tcx>,
    llfn: FunctionValue<'tcx>,
    vars: IndexVec<mir::VarId, LvalueRef<'tcx>>,
    /// map from mir block to llvm block
    blocks: IndexVec<BlockId, BasicBlock<'tcx>>,
}

#[derive(Debug, Clone, Copy)]
struct LvalueRef<'tcx> {
    ptr: PointerValue<'tcx>,
    ty: Ty<'tcx>,
}

#[derive(Debug, Clone, Copy)]
struct ValueRef<'tcx> {
    val: BasicValueEnum<'tcx>,
    ty: Ty<'tcx>,
}

#[derive(Debug, Clone, Copy)]
struct LLVMVar<'tcx> {
    ptr: PointerValue<'tcx>,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn new(
        cctx: &'a CodegenCtx<'tcx>,
        llfn: FunctionValue<'tcx>,
        body: &'tcx mir::Mir<'tcx>,
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

    fn alloc_vars(&mut self) -> IndexVec<VarId, LvalueRef<'tcx>> {
        let alloca = |var_id| {
            let mir_var = self.body.vars[var_id];
            let ty = mir_var.ty;
            let ptr = self.build_alloca(self.llvm_ty(ty), &mir_var.to_string());
            LvalueRef { ptr, ty }
        };

        // store arguments into the respective vars
        let args = self.body.arg_iter().zip(self.llfn.get_param_iter()).map(|(id, llval)| {
            let var = alloca(id);
            // store the provided arguments into the local variables we provided
            self.build_store(var.ptr, llval);
            var
        });

        let retvar = alloca(VarId::new(mir::RETURN));
        let vars = self.body.var_iter().map(alloca);
        std::iter::once(retvar).chain(args).chain(vars).collect()
    }

    /// entry point of `FnCtx`
    crate fn codegen(&mut self) {
        for id in self.body.basic_blocks.indices() {
            self.codegen_basic_block(id);
        }
        // self.fpm.run_on(&self.llfn);
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

    /// generates the code to retrieve the reference count for a given variable
    /// the variable must refer to a box to be valid
    /// returns the i32 pointer to the refcount itself
    fn build_get_rc(&mut self, var: VarId) -> PointerValue<'tcx> {
        let lvalue = self.vars[var];
        // `ptr` is a pointer into the boxed content (not including the rc header)
        let ptr = self.build_load(lvalue.ptr, "load_box").into_pointer_value();
        // we need to deref as `lvalue.ty` is the type of the pointer to the box
        let llty = self.llvm_boxed_ty(lvalue.ty.deref_ty()).ptr_type(AddressSpace::Generic);
        let cast = self.build_pointer_cast(ptr, llty, "rc_cast");
        // finally the reference count itself
        self.build_struct_gep(cast, 1, "rc").unwrap()
    }

    fn codegen_stmt(&mut self, stmt: &'tcx mir::Stmt<'tcx>) {
        match stmt.kind {
            mir::StmtKind::Assign(lvalue, ref rvalue) => self.codegen_assignment(lvalue, rvalue),
            mir::StmtKind::Retain(var) => {
                let rc = self.build_get_rc(var);
                self.build_atomicrmw(
                    AtomicRMWBinOp::Add,
                    rc,
                    self.vals.one32,
                    AtomicOrdering::SequentiallyConsistent,
                )
                .unwrap();
            }
            mir::StmtKind::Release(var_id) => {
                let ptr = self.vars[var_id].ptr;
                // we cast it pointer to an i8* as that is what `rc_release` expects
                let cast = self.build_pointer_cast(ptr, self.types.i8ptr, "rc_release_cast").into();
                let rc = self.build_get_rc(var_id);
                self.build_call(self.native_functions.rc_release, &[cast, rc.into()], "rc_release");
            }
            mir::StmtKind::Nop => {}
        }
    }

    fn codegen_assignment(&mut self, lvalue: mir::Lvalue<'tcx>, rvalue: &'tcx mir::Rvalue<'tcx>) {
        let var = self.codegen_lvalue(lvalue);
        // certain aggregate rvalues require special treatment as
        // llvm doesn't like recursively building these values (with temporaries)
        // instead, we use geps to set the fields directly
        match rvalue {
            mir::Rvalue::Adt { adt, substs, fields, variant_idx } => {
                let ty = self.tcx.mk_adt_ty(adt, substs);
                match adt.kind {
                    // basically identical code to tuple but has potential substs to deal with
                    AdtKind::Struct => {
                        assert_eq!(variant_idx.index(), 0);
                        for (i, f) in fields.iter().enumerate() {
                            let operand = self.codegen_operand(f);
                            let field_ptr =
                                self.build_struct_gep(var.ptr, i as u32, "struct_gep").unwrap();
                            self.build_store(field_ptr, operand.val);
                        }
                    }
                    AdtKind::Enum => {
                        let idx = variant_idx.index() as u64;
                        let discr_ptr = self.build_struct_gep(var.ptr, 0, "discr_gep").unwrap();
                        self.build_store(discr_ptr, self.types.discr.const_int(idx, false));
                        let content_ptr = self.build_struct_gep(var.ptr, 1, "enum_gep").unwrap();
                        let llty = self.variant_ty_to_llvm_ty(&adt.variants[*variant_idx], substs);
                        let content_ptr = self.build_pointer_cast(
                            content_ptr,
                            llty.ptr_type(AddressSpace::Generic),
                            "enum_ptr_cast",
                        );
                        for (i, f) in fields.iter().enumerate() {
                            let operand = self.codegen_operand(f);
                            let field_ptr = self
                                .build_struct_gep(content_ptr, i as u32, "enum_content_gep")
                                .unwrap();
                            self.build_store(field_ptr, operand.val);
                        }
                    }
                }
            }
            _ => {
                let value = self.codegen_rvalue(rvalue);
                self.build_store(var.ptr, value.val);
            }
        }
    }

    /// returns a pointer to where the lvalue points to
    fn codegen_lvalue(&mut self, lvalue: mir::Lvalue<'tcx>) -> LvalueRef<'tcx> {
        self.codegen_lvalue_inner(lvalue.id, lvalue.projs.as_ref())
    }

    fn codegen_lvalue_inner(
        &mut self,
        var_id: VarId,
        projs: &[Projection<'tcx>],
    ) -> LvalueRef<'tcx> {
        match projs {
            [] => self.vars[var_id],
            [projs @ .., proj] => {
                // recursively process all the projections to the left
                let var = self.codegen_lvalue_inner(var_id, projs);
                match proj {
                    Projection::Field(f, ty) => {
                        let index = f.index() as u32;
                        let ptr = self.build_struct_gep(var.ptr, index, "struct_gep").unwrap();
                        LvalueRef { ptr, ty }
                    }
                    Projection::Deref => {
                        let ptr = self.build_load(var.ptr, "load_deref").into_pointer_value();
                        let ty = var.ty.deref_ty();
                        LvalueRef { ptr, ty }
                    }
                }
            }
        }
    }

    /// saves the previous insert block, runs a function, then restores the builder to the end of
    /// the previous basic block
    fn with_new_insertion_point<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        // save the insertion point of the outer function
        let prev_block = self.get_insert_block();
        let ret = f(self);
        if let Some(block) = prev_block {
            self.position_at_end(block);
        }
        ret
    }

    fn codegen_rvalue(&mut self, rvalue: &'tcx mir::Rvalue<'tcx>) -> ValueRef<'tcx> {
        match rvalue {
            mir::Rvalue::Closure(ty, body) => {
                let name = "<closure>";
                let f = self.cctx.module.add_function(name, self.llvm_fn_ty_from_ty(ty), None);
                self.with_new_insertion_point(|ctx| ctx.codegen_body(name, body));
                let val = f.as_llvm_ptr().into();
                ValueRef { val, ty }
            }
            mir::Rvalue::Operand(operand) => self.codegen_operand(operand),
            mir::Rvalue::Box(ty) => {
                // important the refcount itself is boxed so it is shared
                let llty = self.llvm_boxed_ty(ty);
                let ptr = self.build_malloc(llty, "box").unwrap();
                // the refcount is at index `1` in the implicit struct
                let rc = self.build_struct_gep(ptr, 1, "rc").unwrap();
                // set the rc to `1` initially
                self.build_atomicrmw(
                    AtomicRMWBinOp::Xchg,
                    rc,
                    self.vals.one32,
                    AtomicOrdering::SequentiallyConsistent,
                )
                .unwrap();
                // gep the returned pointer to point to the content only and return that
                let val = self.build_struct_gep(ptr, 0, "rc_gep").unwrap().into();
                ValueRef { ty, val }
            }
            mir::Rvalue::Ref(lvalue) => {
                // ValueRef { val: self.codegen_lvalue(*lvalue).ptr.into(), ty: todo!() },
                panic!("unsupported");
            }
            mir::Rvalue::Bin(op, l, r) => {
                let lhs = self.codegen_operand(l);
                let rhs = self.codegen_operand(r);
                match (lhs.val, rhs.val) {
                    (BasicValueEnum::FloatValue(_), BasicValueEnum::FloatValue(_)) =>
                        self.codegen_float_op(*op, lhs, rhs),
                    (BasicValueEnum::IntValue(_), BasicValueEnum::IntValue(_)) =>
                        self.codegen_int_op(*op, lhs, rhs),
                    _ => unreachable!(),
                }
            }
            mir::Rvalue::Unary(_, _) => todo!(),
            // handle these cases in `codegen_assignment`
            mir::Rvalue::Adt { .. } => unreachable!(),
        }
    }

    fn codegen_operand(&mut self, operand: &mir::Operand<'tcx>) -> ValueRef<'tcx> {
        match operand {
            mir::Operand::Const(c) => match c.kind {
                ConstKind::Float(f) => ValueRef {
                    val: self.types.float.const_float(f).into(),
                    ty: self.tcx.types.float,
                },
                ConstKind::Int(i) => ValueRef {
                    val: self.types.int.const_int(i as u64, true).into(),
                    ty: self.tcx.types.int,
                },
                ConstKind::Bool(b) => ValueRef {
                    val: self.types.boolean.const_int(b as u64, true).into(),
                    ty: self.tcx.types.boolean,
                },
                ConstKind::Unit => ValueRef { val: self.vals.unit.into(), ty: self.tcx.types.unit },
            },
            &mir::Operand::Lvalue(lvalue) => {
                let var = self.codegen_lvalue(lvalue);
                let val = self.build_load(var.ptr, "load").into();
                ValueRef { val, ty: var.ty }
            }
            mir::Operand::Item(def_id) => {
                // TODO assume item is fn for now
                let ident = self
                    .items
                    .borrow()
                    .get(def_id)
                    .copied()
                    .unwrap_or_else(|| panic!("no entry in items with def_id `{}`", def_id));
                let llfn = self.module.get_function(ident.as_str()).unwrap();
                // probably not the `correct` way to do this :)
                let val =
                    unsafe { std::mem::transmute::<FunctionValue, PointerValue>(llfn) }.into();
                ValueRef { val, ty: self.tcx.collected_ty(*def_id) }
            }
        }
    }

    fn codegen_int_op(
        &mut self,
        op: BinOp,
        lhs: ValueRef<'tcx>,
        rhs: ValueRef<'tcx>,
    ) -> ValueRef<'tcx> {
        let l = lhs.val.into_int_value();
        let r = rhs.val.into_int_value();
        let val = match op {
            BinOp::Mul => self.build_int_mul(l, r, "imul").into(),
            BinOp::Div => self.build_int_signed_div(l, r, "idiv").into(),
            BinOp::Add => self.build_int_add(l, r, "iadd").into(),
            BinOp::Sub => self.build_int_sub(l, r, "isub").into(),
            BinOp::And => self.build_and(l, r, "and").into(),
            BinOp::Or => self.build_or(l, r, "or").into(),
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt =>
                return self.compile_icmp(op, lhs, rhs),
        };
        assert_eq!(lhs.ty, rhs.ty);
        ValueRef { val, ty: self.tcx.types.int }
    }

    fn codegen_float_op(
        &mut self,
        op: BinOp,
        lhs: ValueRef<'tcx>,
        rhs: ValueRef<'tcx>,
    ) -> ValueRef<'tcx> {
        let l = lhs.val.into_float_value();
        let r = rhs.val.into_float_value();
        let val = match op {
            BinOp::Mul => self.build_float_mul(l, r, "tmpfmul"),
            BinOp::Div => self.build_float_div(l, r, "tmpfdiv"),
            BinOp::Add => self.build_float_add(l, r, "tmpadd"),
            BinOp::Sub => self.build_float_sub(l, r, "tmpfsub"),
            BinOp::And | BinOp::Or => unreachable!(),
            BinOp::Lt | BinOp::Gt | BinOp::Eq | BinOp::Neq =>
                return self.compile_fcmp(op, lhs, rhs),
        };
        assert_eq!(lhs.ty, rhs.ty);
        ValueRef { val: val.into(), ty: self.tcx.types.float }
    }

    fn compile_icmp(
        &mut self,
        op: BinOp,
        lhs: ValueRef<'tcx>,
        rhs: ValueRef<'tcx>,
    ) -> ValueRef<'tcx> {
        let l = lhs.val.into_int_value();
        let r = rhs.val.into_int_value();
        let val = match op {
            BinOp::Lt => self.builder.build_int_compare(IntPredicate::SLT, l, r, "icmp_lt"),
            BinOp::Gt => self.builder.build_int_compare(IntPredicate::SGT, l, r, "icmp_gt"),
            BinOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, l, r, "icmp_eq"),
            BinOp::Neq => self.build_int_compare(IntPredicate::NE, l, r, "icmp_neq"),
            BinOp::And | BinOp::Or | BinOp::Mul | BinOp::Div | BinOp::Add | BinOp::Sub =>
                unreachable!(),
        };
        assert_eq!(lhs.ty, rhs.ty);
        ValueRef { val: val.into(), ty: self.tcx.types.boolean }
    }

    fn compile_fcmp(
        &mut self,
        op: BinOp,
        lhs: ValueRef<'tcx>,
        rhs: ValueRef<'tcx>,
    ) -> ValueRef<'tcx> {
        let l = lhs.val.into_float_value();
        let r = lhs.val.into_float_value();
        let val = match op {
            BinOp::Lt => self.builder.build_float_compare(FloatPredicate::OLT, l, r, "fcmp_lt"),
            BinOp::Gt => self.builder.build_float_compare(FloatPredicate::OGT, l, r, "fcmp_gt"),
            BinOp::Eq => self.build_float_compare(FloatPredicate::OEQ, l, r, "fcmp_oeq"),
            BinOp::Neq => self.build_float_compare(FloatPredicate::UNE, l, r, "fcmp_une"),
            BinOp::And | BinOp::Or | BinOp::Mul | BinOp::Div | BinOp::Add | BinOp::Sub =>
                unreachable!(),
        };
        assert_eq!(l, r);
        ValueRef { val: val.into(), ty: self.tcx.types.boolean }
    }

    fn codegen_terminator(&mut self, terminator: &mir::Terminator<'tcx>) {
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
                let f = self.codegen_operand(f).val.into_pointer_value();
                let args = args.iter().map(|arg| self.codegen_operand(arg).val).collect_vec();
                let value = self.build_call(f, &args, "fcall").try_as_basic_value().left().unwrap();
                let lvalue_ref = self.codegen_lvalue(*lvalue);
                self.build_store(lvalue_ref.ptr, value);
                self.build_unconditional_branch(self.blocks[*target]);
            }
            mir::TerminatorKind::Switch { discr, arms, default } =>
                self.codegen_switch(discr, arms, *default),
            mir::TerminatorKind::Cond(cond, then, els) => {
                let cond = self.codegen_operand(cond);
                self.build_conditional_branch(
                    cond.val.into_int_value(),
                    self.blocks[*then],
                    self.blocks[*els],
                );
            }
        }
    }

    fn codegen_switch(
        &mut self,
        discr: &mir::Operand<'tcx>,
        arms: &[(mir::Operand<'tcx>, BlockId)],
        default: BlockId,
    ) {
        let discr = self.codegen_operand(discr).val.into_int_value();
        let arms = arms
            .iter()
            .map(|&(ref rvalue, block)| {
                let rvalue = self.codegen_operand(rvalue).val.into_int_value();
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

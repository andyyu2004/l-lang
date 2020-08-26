use super::util::LLVMAsPtrVal;
use super::FnCtx;
use crate::ast;
use crate::ir::{self, DefId};
use crate::lexer::symbol;
use crate::mir::{self, *};
use crate::tir;
use crate::ty::{Const, ConstKind, SubstsRef, Ty, TyKind};
use crate::typeck::TyCtx;
use inkwell::types::{BasicType, BasicTypeEnum, FloatType, FunctionType};
use inkwell::values::*;
use inkwell::{
    basic_block::BasicBlock, builder::Builder, context::Context, module::Module, passes::PassManager
};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::Display;

pub struct CodegenCtx<'tcx> {
    pub tcx: TyCtx<'tcx>,
    pub llctx: &'tcx Context,
    pub builder: Builder<'tcx>,
    pub fpm: PassManager<FunctionValue<'tcx>>,
    pub module: Module<'tcx>,
    pub vals: CommonValues<'tcx>,
    curr_fn: Option<FunctionValue<'tcx>>,
}

pub struct CommonValues<'tcx> {
    zero: IntValue<'tcx>,
}

impl<'tcx> CodegenCtx<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, ctx: &'tcx Context) -> Self {
        let module = ctx.create_module("main");
        let fpm = PassManager::create(&module);
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_basic_alias_analysis_pass();
        fpm.add_promote_memory_to_register_pass();
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.initialize();
        let vals = CommonValues { zero: ctx.i64_type().const_zero() };
        Self { tcx, llctx: ctx, module, fpm, builder: ctx.create_builder(), vals, curr_fn: None }
    }

    /// returns the main function
    pub fn compile(&mut self, prog: &'tcx mir::Prog<'tcx>) -> FunctionValue<'tcx> {
        let main = prog.bodies.values().next().unwrap();
        let llvm_fn = self.compile_fn(main);
        self.module.verify().unwrap();
        self.module.print_to_file("ir.ll").unwrap();
        llvm_fn
    }

    fn compile_fn(&mut self, body: &'tcx mir::Body<'tcx>) -> FunctionValue<'tcx> {
        let tmp_fn_ty = self.llvm_ty(self.tcx.types.num).fn_type(&[], false);
        let llvm_fn = self.module.add_function("main", tmp_fn_ty, None);
        let mut fcx = FnCtx::new(&self, body, llvm_fn);
        fcx.codegen_body(body);
        llvm_fn.verify(true);
        llvm_fn
    }

    pub fn llvm_fn_ty(&self, params: SubstsRef, ret: Ty) -> FunctionType<'tcx> {
        self.llvm_ty(ret)
            .fn_type(&params.into_iter().map(|ty| self.llvm_ty(ty)).collect_vec(), false)
    }

    pub fn llvm_ty(&self, ty: Ty) -> BasicTypeEnum<'tcx> {
        match ty.kind {
            TyKind::Bool => self.llctx.bool_type().into(),
            TyKind::Char => todo!(),
            TyKind::Num => self.llctx.f64_type().into(),
            TyKind::Array(ty) => todo!(),
            TyKind::Fn(params, ret) =>
                self.llvm_fn_ty(params, ret).ptr_type(AddressSpace::Generic).into(),
            TyKind::Tuple(_) => todo!(),
            TyKind::Param(_) => todo!(),
            TyKind::Scheme(_, _) => todo!(),
            TyKind::Never => todo!(),
            TyKind::Error | TyKind::Infer(_) => unreachable!(),
            TyKind::Adt(..) => todo!(),
        }
    }
}

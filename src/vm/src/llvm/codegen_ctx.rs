use super::util::LLVMAsPtrVal;
use super::FnCtx;
use crate::ast;
use crate::error::{LLVMError, LResult};
use crate::ir::{self, DefId};
use crate::lexer::symbol;
use crate::mir::{self, *};
use crate::span::Span;
use crate::tir;
use crate::ty::{AdtKind, Const, ConstKind, SubstsRef, Ty, TyKind};
use crate::typeck::TyCtx;
use ast::Ident;
use inkwell::types::*;
use inkwell::values::*;
use inkwell::{
    basic_block::BasicBlock, builder::Builder, context::Context, module::Module, passes::PassManager
};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Deref;
use symbol::Symbol;

pub struct CodegenCtx<'tcx> {
    pub tcx: TyCtx<'tcx>,
    pub llctx: &'tcx Context,
    pub builder: Builder<'tcx>,
    pub fpm: PassManager<FunctionValue<'tcx>>,
    pub module: Module<'tcx>,
    pub vals: CommonValues<'tcx>,
    pub types: CommonTypes<'tcx>,
    /// stores the `Ident` for a `DefId` which can then be used to lookup the function in the `llctx`
    /// this api is a bit awkward, but its what inkwell has so..
    pub items: RefCell<FxHashMap<DefId, Ident>>,
    pub lltypes: RefCell<FxHashMap<Ty<'tcx>, BasicTypeEnum<'tcx>>>,
    curr_fn: Option<FunctionValue<'tcx>>,
}

pub struct CommonValues<'tcx> {
    pub zero: IntValue<'tcx>,
    pub unit: StructValue<'tcx>,
}

pub struct CommonTypes<'tcx> {
    pub unit: StructType<'tcx>,
    pub int: IntType<'tcx>,
    pub float: FloatType<'tcx>,
    pub boolean: IntType<'tcx>,
}

impl<'tcx> CodegenCtx<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, llctx: &'tcx Context) -> Self {
        let module = llctx.create_module("main");
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
        let types = CommonTypes {
            unit: llctx.struct_type(&[], false),
            int: llctx.i64_type(),
            float: llctx.f64_type(),
            boolean: llctx.bool_type(),
        };
        let vals =
            CommonValues { zero: llctx.i64_type().const_zero(), unit: types.unit.get_undef() };

        // assert `unit` is indeed a zst
        assert_eq!(vals.unit.get_type().size_of().unwrap().print_to_string().to_string(), "i64 0");

        Self {
            tcx,
            llctx,
            module,
            fpm,
            builder: llctx.create_builder(),
            vals,
            types,
            curr_fn: None,
            items: Default::default(),
            lltypes: Default::default(),
        }
    }

    /// returns the main function
    pub fn codegen(&mut self, prog: &'tcx mir::Prog<'tcx>) -> Option<FunctionValue<'tcx>> {
        // we need to predeclare all items as we don't require them to be declared in the source
        // file in topological order
        for (id, item) in &prog.items {
            self.items.borrow_mut().insert(id.def, item.ident);
            match &item.kind {
                ItemKind::Fn(body) => {
                    let (_, ty) = self.tcx.item_ty(id.def).expect_scheme();
                    let (params, ret) = ty.expect_fn();
                    let llty = self.llvm_fn_ty(params, ret);
                    let llfn = self.module.add_function(item.ident.as_str(), llty, None);
                }
            };
        }
        for (id, item) in &prog.items {
            match &item.kind {
                ItemKind::Fn(body) => self.codegen_body(item, body),
            };
        }
        self.module.print_to_stderr();
        self.module.print_to_file("ir.ll").unwrap();
        self.module.verify().unwrap();
        self.module.get_function(symbol::MAIN.as_str()).or_else(|| {
            self.tcx.sess.build_error(Span::empty(), LLVMError::MissingMain).emit();
            None
        })
    }

    fn codegen_body(
        &mut self,
        item: &mir::Item,
        body: &'tcx mir::Body<'tcx>,
    ) -> FunctionValue<'tcx> {
        let llfn = self.module.get_function(item.ident.as_str()).unwrap();
        let mut fcx = FnCtx::new(&self, body, llfn);
        fcx.codegen_body();
        llfn
    }

    pub fn llvm_fn_ty(&self, params: SubstsRef<'tcx>, ret: Ty<'tcx>) -> FunctionType<'tcx> {
        self.llvm_ty(ret).fn_type(&params.iter().map(|ty| self.llvm_ty(ty)).collect_vec(), false)
    }

    /// converts a L type into a llvm representation
    pub fn llvm_ty(&self, ty: Ty<'tcx>) -> BasicTypeEnum<'tcx> {
        if let Some(&llty) = self.lltypes.borrow().get(ty) {
            return llty;
        }
        let llty = match ty.kind {
            TyKind::Bool => self.types.boolean.into(),
            TyKind::Int => self.types.int.into(),
            TyKind::Float => self.types.float.into(),
            TyKind::Tuple(xs) if xs.is_empty() => self.types.unit.into(),
            TyKind::Char => todo!(),
            TyKind::Array(ty) => todo!(),
            TyKind::Fn(params, ret) =>
                self.llvm_fn_ty(params, ret).ptr_type(AddressSpace::Generic).into(),
            TyKind::Tuple(tys) => {
                // tuples are represented as anonymous structs
                let lltys = tys.iter().map(|ty| self.llvm_ty(ty)).collect_vec();
                self.llctx.struct_type(&lltys, false).into()
            }
            TyKind::Param(_) => todo!(),
            TyKind::Scheme(_, _) => todo!(),
            TyKind::Never => todo!(),
            TyKind::Error | TyKind::Infer(_) => unreachable!(),
            TyKind::Adt(adt, substs) => match adt.kind {
                AdtKind::Struct => {
                    let variant = adt.single_variant();
                    // note we preserve the field declaration order of the struct
                    let tys = variant.fields.iter().map(|f| self.llvm_ty(f.ty)).collect_vec();
                    self.llctx.struct_type(&tys, false).into()
                }
                AdtKind::Enum => todo!(),
            },
            TyKind::Ptr(_, ty) => self.llvm_ty(ty).ptr_type(AddressSpace::Generic).into(),
            TyKind::Opaque(_, _) => todo!(),
        };
        self.lltypes.borrow_mut().insert(ty, llty);
        llty
    }
}

impl<'tcx> Deref for CodegenCtx<'tcx> {
    type Target = Builder<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

use super::codegen::*;
use super::util::{LLVMAsPtrVal, LLVMTypeExt};
use super::FnCtx;
use crate::ast::{self, Ident};
use crate::error::{LLVMError, LResult};
use crate::ir::{self, DefId, FnVisitor, ItemVisitor};
use crate::lexer::symbol::{self, Symbol};
use crate::mir::{self, *};
use crate::span::Span;
use crate::tir;
use crate::ty::*;
use crate::typeck::TyCtx;
use inkwell::passes::PassManager;
use inkwell::types::*;
use inkwell::values::*;
use inkwell::{basic_block::BasicBlock, builder::Builder, context::Context, module::Module};
use inkwell::{AddressSpace, AtomicOrdering, AtomicRMWBinOp, FloatPredicate, IntPredicate};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Deref;

pub struct CodegenCtx<'tcx> {
    pub tcx: TyCtx<'tcx>,
    pub llctx: &'tcx Context,
    pub fpm: PassManager<FunctionValue<'tcx>>,
    pub module: Module<'tcx>,
    pub vals: CommonValues<'tcx>,
    pub types: CommonTypes<'tcx>,
    pub builder: Builder<'tcx>,
    pub native_functions: NativeFunctions<'tcx>,
    /// stores the `Ident` for a `DefId` which can then be used to lookup the function in the `llctx`
    /// this api is a bit awkward, but its what inkwell has so..
    pub items: RefCell<FxHashMap<DefId, Ident>>,
    pub lltypes: RefCell<FxHashMap<Ty<'tcx>, BasicTypeEnum<'tcx>>>,
}

pub struct CommonValues<'tcx> {
    pub zero: IntValue<'tcx>,
    pub one: IntValue<'tcx>,
    pub neg_one: IntValue<'tcx>,
    pub unit: StructValue<'tcx>,
}

pub struct CommonTypes<'tcx> {
    pub int: IntType<'tcx>,
    pub unit: StructType<'tcx>,
    pub byte: IntType<'tcx>,
    pub float: FloatType<'tcx>,
    pub boolean: IntType<'tcx>,
    pub intptr: PointerType<'tcx>,
    // using a fix sized discriminant for ease for now
    pub discr: IntType<'tcx>,
}

pub struct NativeFunctions<'tcx> {
    pub rc_release: FunctionValue<'tcx>,
}

impl<'tcx> NativeFunctions<'tcx> {
    pub fn new(llctx: &'tcx Context, module: &Module<'tcx>) -> Self {
        let rc_release = module.add_function(
            "rc_release",
            llctx
                .void_type()
                .fn_type(&[llctx.i64_type().ptr_type(AddressSpace::Generic).into()], false),
            None,
        );

        // build the function
        let builder = llctx.create_builder();
        let block = llctx.append_basic_block(rc_release, "rc_release");
        // this should be a pointer to the refcount itself
        let ptr = rc_release.get_first_param().unwrap().into_pointer_value();
        builder.position_at_end(block);
        let one = llctx.i64_type().const_int(1, false);
        let ref_count = builder
            .build_atomicrmw(AtomicRMWBinOp::Sub, ptr, one, AtomicOrdering::SequentiallyConsistent)
            .unwrap();
        let then = llctx.append_basic_block(rc_release, "free");
        let els = llctx.append_basic_block(rc_release, "ret");
        // this ref_count is the count before decrement
        // if refcount == 1 then this the last reference and we can free it
        let cmp = builder.build_int_compare(IntPredicate::EQ, ref_count, one, "rc_cmp");
        builder.build_conditional_branch(cmp, then, els);
        // build trivial else branch
        builder.position_at_end(els);
        builder.build_return(None);

        // build code to free the ptr
        builder.position_at_end(then);
        // conveniently, the pointer passed to free does not need to be the
        // same type as the one given during the malloc call (I think)
        // if it's anything like C, then malloc takes a void pointer
        // but it must be the same address
        builder.build_free(ptr);
        builder.build_return(None);
        Self { rc_release }
    }
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
            unit: llctx.struct_type(&[], true),
            int: llctx.i64_type(),
            float: llctx.f64_type(),
            byte: llctx.i8_type(),
            boolean: llctx.bool_type(),
            intptr: llctx.i64_type().ptr_type(AddressSpace::Generic),
            discr: llctx.i8_type(),
        };
        let vals = CommonValues {
            zero: types.int.const_zero(),
            one: types.int.const_int(1, false),
            neg_one: types.int.const_all_ones(),
            unit: types.unit.get_undef(),
        };

        let native_functions = NativeFunctions::new(llctx, &module);

        Self {
            tcx,
            llctx,
            module,
            fpm,
            vals,
            types,
            native_functions,
            builder: llctx.create_builder(),
            items: Default::default(),
            lltypes: Default::default(),
        }
    }

    pub fn declare_items(&self) {
        // we need to predeclare all items as we don't require them to be declared in the source
        // file in topological order
        DeclarationCollector { cctx: self }.visit_ir(self.tcx.ir);
    }

    pub fn codegen_items(&self) {
        // we need to predeclare all items as we don't require them to be declared in the source
        // file in topological order
        CodegenCollector { cctx: self }.visit_ir(self.tcx.ir);
    }

    /// returns the main function
    pub fn codegen(&mut self) -> Option<FunctionValue<'tcx>> {
        self.declare_items();
        self.codegen_items();
        self.module.print_to_stderr();
        self.module.print_to_file("ir.ll").unwrap();
        self.module.verify().unwrap();
        self.module.get_function(symbol::MAIN.as_str()).or_else(|| {
            self.tcx.sess.build_error(Span::empty(), LLVMError::MissingMain).emit();
            None
        })
    }

    pub fn codegen_body(&self, fn_name: &str, body: &'tcx mir::Body<'tcx>) -> FunctionValue<'tcx> {
        let llfn = self.module.get_function(fn_name).unwrap();
        let mut fcx = FnCtx::new(&self, llfn, body);
        fcx.codegen();
        llfn
    }

    pub fn llvm_fn_ty_from_ty(&self, ty: Ty<'tcx>) -> FunctionType<'tcx> {
        let (params, ret) = ty.expect_fn();
        self.llvm_ty(ret).fn_type(&params.iter().map(|ty| self.llvm_ty(ty)).collect_vec(), false)
    }

    pub fn llvm_fn_ty(&self, params: SubstsRef<'tcx>, ret: Ty<'tcx>) -> FunctionType<'tcx> {
        self.llvm_ty(ret).fn_type(&params.iter().map(|ty| self.llvm_ty(ty)).collect_vec(), false)
    }

    /// wraps a `Ty` with refcount info
    pub fn llvm_boxed_ty(&self, ty: Ty<'tcx>) -> StructType<'tcx> {
        let ty = self.llvm_ty(ty);
        self.llctx.struct_type(&[self.types.int.into(), ty], true)
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
            TyKind::Array(ty, n) => todo!(),
            TyKind::Fn(params, ret) =>
                self.llvm_fn_ty(params, ret).ptr_type(AddressSpace::Generic).into(),
            TyKind::Tuple(tys) => {
                // tuples are represented as anonymous structs
                let lltys = tys.iter().map(|ty| self.llvm_ty(ty)).collect_vec();
                self.llctx.struct_type(&lltys, true).into()
            }
            TyKind::Param(_) => todo!(),
            TyKind::Scheme(_, _) => todo!(),
            TyKind::Never => todo!(),
            TyKind::Error | TyKind::Infer(_) => unreachable!(),
            TyKind::Adt(adt, substs) => match adt.kind {
                AdtKind::Struct => {
                    let variant = adt.single_variant();
                    self.variant_ty_to_llvm_ty(variant, substs).into()
                }
                AdtKind::Enum => {
                    // it is fine to unwrap here as if the enum has no variants it is not
                    // constructable and this will never be called
                    let largest_variant = adt.variants.iter().max_by(|s, t| {
                        self.variant_size(s, substs).cmp(&self.variant_size(t, substs))
                    });
                    let llvariant =
                        self.variant_ty_to_llvm_ty(largest_variant.unwrap(), substs).into();
                    assert!(adt.variants.len() < 256, "too many variants");
                    self.llctx.struct_type(&[self.types.discr.into(), llvariant], false).into()
                }
            },
            TyKind::Ptr(_, ty) => self.llvm_ty(ty).ptr_type(AddressSpace::Generic).into(),
            TyKind::Opaque(..) => unreachable!(),
        };
        self.lltypes.borrow_mut().insert(ty, llty);
        llty
    }

    fn variant_size(&self, variant_ty: &'tcx VariantTy<'tcx>, substs: SubstsRef<'tcx>) -> usize {
        variant_ty.fields.iter().map(|f| f.ty(self.tcx, substs)).map(|ty| self.ty_size(ty)).sum()
    }

    // there is probably all sorts of problems with this
    // as it does not account for padding etc
    // however, this does not need to be exact
    // as this is only used to decide the largest variant in an enum
    // so hopefully its accurate enough for that
    fn ty_size(&self, ty: Ty<'tcx>) -> usize {
        let size = match ty.kind {
            Bool | Char => 1,
            Float | Int => 8,
            // assuming 64bit :P
            Ptr(..) => 8,
            Never => 0,
            Array(ty, n) => n * self.ty_size(ty),
            Fn(_, _) => todo!(),
            Param(_) => todo!(),
            Adt(_, _) => todo!(),
            Scheme(_, _) => todo!(),
            Opaque(_, _) => todo!(),
            Error => unreachable!(),
            Infer(_) => unreachable!(),
            Tuple(tys) => tys.iter().map(|ty| self.ty_size(ty)).sum(),
        };
        info!("sizeof({}) = {}", ty, size);
        size
    }

    pub fn variant_ty_to_llvm_ty(
        &self,
        variant: &VariantTy<'tcx>,
        substs: SubstsRef<'tcx>,
    ) -> StructType<'tcx> {
        // TODO cache results
        // note we preserve the field declaration order of the struct
        let tys = variant.fields.iter().map(|f| self.llvm_ty(f.ty(self.tcx, substs))).collect_vec();
        self.llctx.struct_type(&tys, true)
    }
}

impl<'tcx> Deref for CodegenCtx<'tcx> {
    type Target = Builder<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

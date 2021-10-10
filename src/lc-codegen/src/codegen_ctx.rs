use super::*;
use context::Context;
use inkwell::passes::PassManager;
use inkwell::types::*;
use inkwell::values::*;
use inkwell::*;
use inkwell::{builder::Builder, module::Module};
use lc_core::ty::*;
use lc_error::{ErrorReported, LResult};
use lc_span::{sym, Span};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
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
    pub llvm_intrinsics: LLVMIntrinsics<'tcx>,
    pub gc_functions: GCFunctions<'tcx>,
    pub intrinsics: RefCell<FxHashMap<Instance<'tcx>, FunctionValue<'tcx>>>,
    pub instances: RefCell<FxHashMap<Instance<'tcx>, FunctionValue<'tcx>>>,
    pub lltypes: RefCell<FxHashMap<Ty<'tcx>, BasicTypeEnum<'tcx>>>,
}

pub struct CommonValues<'tcx> {
    pub zero: IntValue<'tcx>,
    pub one: IntValue<'tcx>,
    pub neg_one: IntValue<'tcx>,
    pub zero32: IntValue<'tcx>,
    pub one32: IntValue<'tcx>,
    pub neg_one32: IntValue<'tcx>,
    pub unit: StructValue<'tcx>,
}

pub struct CommonTypes<'tcx> {
    pub i32: IntType<'tcx>,
    pub i64: IntType<'tcx>,
    pub unit: StructType<'tcx>,
    pub byte: IntType<'tcx>,
    pub float: FloatType<'tcx>,
    pub bool: IntType<'tcx>,
    pub i8ptr: PointerType<'tcx>,
    pub i32ptr: PointerType<'tcx>,
    pub i64ptr: PointerType<'tcx>,
    // using a fix sized discriminant for ease for now
    pub discr: IntType<'tcx>,
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
            i64: llctx.i64_type(),
            i32: llctx.i32_type(),
            float: llctx.f64_type(),
            byte: llctx.i8_type(),
            bool: llctx.bool_type(),
            i8ptr: llctx.i8_type().ptr_type(AddressSpace::Generic),
            i32ptr: llctx.i32_type().ptr_type(AddressSpace::Generic),
            i64ptr: llctx.i64_type().ptr_type(AddressSpace::Generic),
            discr: llctx.i16_type(),
        };

        let vals = CommonValues {
            zero: types.i64.const_zero(),
            one: types.i64.const_int(1, false),
            neg_one: types.i64.const_all_ones(),
            zero32: types.i32.const_zero(),
            one32: types.i32.const_int(1, false),
            neg_one32: types.i32.const_all_ones(),
            unit: types.unit.get_undef(),
        };

        let native_functions = NativeFunctionsBuilder::new(llctx, &module).build();
        let llvm_intrinsics = LLVMIntrinsics::new(llctx, &module);
        let gc = GCFunctions::new(llctx, &module);

        Self {
            tcx,
            llctx,
            module,
            fpm,
            vals,
            types,
            llvm_intrinsics,
            native_functions,
            gc_functions: gc,
            builder: llctx.create_builder(),
            intrinsics: Default::default(),
            instances: Default::default(),
            lltypes: Default::default(),
        }
    }

    pub fn declare_instances<I>(&self, instances: &I)
    where
        for<'a> &'a I: IntoIterator<Item = &'a Instance<'tcx>>,
    {
        instances.into_iter().for_each(|&instance| self.declare_instance(instance));
    }

    fn declare_instance(&self, instance: Instance<'tcx>) {
        match instance.kind {
            InstanceKind::Item => {
                let Instance { def_id, substs, .. } = instance;
                let ty = self.tcx.mk_fn_ptr(self.tcx.fn_sig(def_id));
                let ident = self.tcx.defs().ident(def_id);
                // we need a special case with main, as the name actually matters
                // for lli etc
                let name = if ident.symbol == sym::main {
                    let span = self.tcx.defs().span(def_id);
                    if self.module.get_function(sym::main.as_str()).is_some() {
                        self.tcx.sess.emit_error(span, LLVMError::DuplicateMain);
                    }
                    if ty != self.tcx.types.main {
                        self.tcx.sess.emit_error(span, LLVMError::InvalidMainType(ty));
                    }
                    // `clang bitcode.bc` expects `main` symbol
                    // `ld` itself expects `_start`
                    ident.to_string()
                } else {
                    format!("{}<{}>", ident, substs)
                };
                let llty = self.llvm_fn_ty_from_ty(ty.subst(self.tcx, substs));
                let llfn = self.module.add_function(&name, llty, None);
                self.instances
                    .borrow_mut()
                    .insert(Instance::resolve(self.tcx, def_id, substs), llfn);
            }
            InstanceKind::Intrinsic => self.codegen_intrinsic(instance),
        }
    }

    pub fn codegen_instances(&self) {
        self.instances.borrow().keys().for_each(|&instance| self.codegen_instance(instance));
    }

    pub fn codegen_instance(&self, instance: Instance<'tcx>) {
        match instance.kind {
            InstanceKind::Item => FnCtx::new(self, instance).codegen(),
            // codegenned during declaration
            InstanceKind::Intrinsic => {}
        }
    }

    /// returns the main function
    pub fn codegen(&self) -> LResult<()> {
        let instances = self.tcx.monomorphization_instances(());
        if self.tcx.sess.has_errors() {
            return Err(ErrorReported);
        }
        self.declare_instances(instances);
        self.codegen_instances();
        self.module.verify().unwrap();
        if self.module.get_function(sym::main.as_str()).is_none() {
            self.tcx.sess.build_error(Span::default(), LLVMError::MissingMain).emit();
        }
        Ok(())
    }
}

impl<'tcx> Deref for CodegenCtx<'tcx> {
    type Target = Builder<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

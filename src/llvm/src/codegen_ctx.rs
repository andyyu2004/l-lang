use super::*;
use context::{Context, ContextRef};
use inkwell::passes::PassManager;
use inkwell::types::*;
use inkwell::values::*;
use inkwell::*;
use inkwell::{builder::Builder, module::Module};
use ir::{self, DefId, ItemVisitor};
use lcore::mir::Mir;
use lcore::ty::*;
use lcore::TyCtx;
use rustc_hash::FxHashMap;
use span::{sym, Span, Symbol};
use std::cell::RefCell;
use std::ops::Deref;
use typeck::{TcxCollectExt, Typeof};

pub struct CodegenCtx<'tcx> {
    pub tcx: TyCtx<'tcx>,
    pub llctx: &'tcx Context,
    pub fpm: PassManager<FunctionValue<'tcx>>,
    pub module: Module<'tcx>,
    pub vals: CommonValues<'tcx>,
    pub types: CommonTypes<'tcx>,
    pub builder: Builder<'tcx>,
    pub native_functions: NativeFunctions<'tcx>,
    pub intrinsics: FxHashMap<Symbol, FunctionValue<'tcx>>,
    pub items: RefCell<FxHashMap<DefId, FunctionValue<'tcx>>>,
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
    pub int: IntType<'tcx>,
    pub int32: IntType<'tcx>,
    pub unit: StructType<'tcx>,
    pub byte: IntType<'tcx>,
    pub float: FloatType<'tcx>,
    pub boolean: IntType<'tcx>,
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
            int: llctx.i64_type(),
            int32: llctx.i32_type(),
            float: llctx.f64_type(),
            byte: llctx.i8_type(),
            boolean: llctx.bool_type(),
            i8ptr: llctx.i8_type().ptr_type(AddressSpace::Generic),
            i32ptr: llctx.i32_type().ptr_type(AddressSpace::Generic),
            i64ptr: llctx.i64_type().ptr_type(AddressSpace::Generic),
            // this is obviously quite wasteful but whatever for now
            discr: llctx.i64_type(),
        };

        let vals = CommonValues {
            zero: types.int.const_zero(),
            one: types.int.const_int(1, false),
            neg_one: types.int.const_all_ones(),
            zero32: types.int32.const_zero(),
            one32: types.int32.const_int(1, false),
            neg_one32: types.int32.const_all_ones(),
            unit: types.unit.get_undef(),
        };

        let native_functions = NativeFunctions::new(llctx, &module);
        let intrinsics = build_instrinsics(&module);
        Self {
            tcx,
            llctx,
            module,
            fpm,
            vals,
            types,
            native_functions,
            intrinsics,
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
        self.tcx.collect_item_types();
        self.declare_items();
        self.codegen_items();
        // self.module.print_to_stderr();
        self.module.print_to_file("ir.ll").unwrap();
        self.module.verify().unwrap();
        self.module.get_function(sym::MAIN.as_str()).or_else(|| {
            self.tcx.sess.build_error(Span::empty(), LLVMError::MissingMain).emit();
            None
        })
    }

    pub fn codegen_body(&self, fn_name: &str, body: &'tcx Mir<'tcx>) -> FunctionValue<'tcx> {
        let llfn = self.module.get_function(fn_name).unwrap();
        let mut fcx = FnCtx::new(&self, llfn, body);
        fcx.codegen();
        llfn
    }

    fn const_size_of(&self, llty: BasicTypeEnum<'tcx>) {
        let idx = self.types.int32.const_int(1, false);
    }

    pub fn adt_size(&self, adt: &'tcx AdtTy<'tcx>, substs: SubstsRef<'tcx>) -> usize {
        // this works for both enums and structs
        // as structs by definition only have one variant the max is essentially redundant
        adt.variants.iter().map(|v| self.variant_size(v, substs)).max().unwrap()
    }

    pub fn variant_size(
        &self,
        variant_ty: &'tcx VariantTy<'tcx>,
        substs: SubstsRef<'tcx>,
    ) -> usize {
        variant_ty
            .fields
            .iter()
            .map(|f| f.ty(self.tcx, substs))
            .map(|ty| self.approx_sizeof(ty))
            .sum()
    }

    // there is probably all sorts of problems with this
    // as it does not account for padding etc
    // however, this does not need to be exact
    // as this is only used to decide the largest variant in an enum
    // so hopefully its accurate enough for that
    fn approx_sizeof(&self, ty: Ty<'tcx>) -> usize {
        let size = match ty.kind {
            Bool | Char => 1,
            Float | Int => 8,
            // assuming 64bit..
            Ptr(..) => 8,
            Array(ty, n) => n * self.approx_sizeof(ty),
            Fn(..) => 8,
            Adt(adt, substs) => self.adt_size(adt, substs),
            Tuple(tys) => tys.iter().map(|ty| self.approx_sizeof(ty)).sum(),
            Opaque(_, _) => todo!(),
            Scheme(..) | Param(..) | Infer(..) | Never | Error =>
                unreachable!("`approx_sizeof` called on {}", ty),
        };
        size
    }
}

impl<'tcx> Deref for CodegenCtx<'tcx> {
    type Target = Builder<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

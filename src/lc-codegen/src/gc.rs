use crate::llvm_ty;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

// it is important to only each of these intrinsics exactly once
// each new invocation will create a new copy with a suffix integer
pub struct GCFunctions<'tcx> {
    pub gc_malloc: FunctionValue<'tcx>,
    pub gc_malloc_atomic: FunctionValue<'tcx>,
}

impl<'tcx> GCFunctions<'tcx> {
    pub fn new(llctx: &'tcx Context, module: &Module<'tcx>) -> Self {
        let gc_malloc = module.add_function("GC_malloc", llvm_ty!(llctx, fn(i64) -> *i8), None);
        let gc_malloc_atomic =
            module.add_function("GC_malloc_atomic", llvm_ty!(llctx, fn(i64) -> *i8), None);

        Self { gc_malloc, gc_malloc_atomic }
    }
}

use crate::llvm_ty;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

// it is important to only each of these intrinsics exactly once
// each new invocation will create a new copy with a suffix integer
pub struct LLVMIntrinsics<'tcx> {
    pub stackmap: FunctionValue<'tcx>,
}

impl<'tcx> LLVMIntrinsics<'tcx> {
    pub fn new(llctx: &'tcx Context, module: &Module<'tcx>) -> Self {
        let stackmap = module.add_function(
            "llvm.experimental.stackmap",
            llvm_ty!(llctx, dyn fn(i64, i32)),
            None,
        );

        Self { stackmap }
    }
}

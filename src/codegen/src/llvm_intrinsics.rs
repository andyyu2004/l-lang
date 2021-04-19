use crate::{llvm_ty, CodegenCtx};

impl<'tcx> CodegenCtx<'tcx> {
    pub fn stackmap(&self) {
        self.module.add_function(
            "llvm.experimental.gc.statepoint",
            llvm_ty!(self, dyn fn(i64, i32)),
            None,
        );
    }
}

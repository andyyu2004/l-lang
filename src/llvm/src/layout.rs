use crate::CodegenCtx;
use inkwell::types::{AsTypeRef, BasicTypeEnum};
use lcore::ty::Ty;
use llvm_sys::target::*;

impl<'tcx> CodegenCtx<'tcx> {
    pub fn sizeof(&self, llty: BasicTypeEnum<'tcx>) -> u64 {
        let type_ref = llty.as_type_ref();
        let opaque_target_data = unsafe { LLVMGetModuleDataLayout(self.module.get_module_ref()) };
        unsafe { LLVMABISizeOfType(opaque_target_data, type_ref) }
    }

    pub fn sizeof_ty(&self, ty: Ty<'tcx>) -> u64 {
        self.sizeof(self.llvm_ty(ty))
    }
}

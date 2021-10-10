use crate::CodegenCtx;
use inkwell::types::BasicType;
use lc_core::ty::{SubstsRef, Ty, VariantTy};
use llvm_sys::target::*;

impl<'tcx> CodegenCtx<'tcx> {
    pub fn sizeof(&self, llty: impl BasicType<'tcx>) -> u64 {
        let type_ref = llty.as_type_ref();
        let opaque_target_data = unsafe { LLVMGetModuleDataLayout(self.module.get_module_ref()) };
        unsafe { LLVMABISizeOfType(opaque_target_data, type_ref) }
    }

    pub fn sizeof_ty(&self, ty: Ty<'tcx>) -> u64 {
        let size = self.sizeof(self.llvm_ty(ty));
        debug!("sizeof {} {}", ty, size);
        size
    }

    pub fn variant_size(&self, variant_ty: &'tcx VariantTy, substs: SubstsRef<'tcx>) -> u64 {
        variant_ty.fields.iter().map(|f| f.ty(self.tcx, substs)).map(|ty| self.sizeof_ty(ty)).sum()
    }
}

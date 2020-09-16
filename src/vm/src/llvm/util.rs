use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{FunctionValue, PointerValue};
use llvm_sys::core::{LLVMConstIntGetZExtValue, LLVMSizeOf};

pub trait LLVMAsPtrVal<'tcx> {
    fn as_llvm_ptr(self) -> PointerValue<'tcx>;
}

impl<'tcx> LLVMAsPtrVal<'tcx> for FunctionValue<'tcx> {
    fn as_llvm_ptr(self) -> PointerValue<'tcx> {
        unsafe { std::mem::transmute(self) }
    }
}

pub trait LLVMTypeExt<'tcx> {
    fn size(&self) -> u64;
}

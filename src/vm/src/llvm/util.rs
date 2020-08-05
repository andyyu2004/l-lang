use inkwell::values::{FunctionValue, PointerValue};

pub trait LLVMAsPtrVal<'tcx> {
    fn as_llvm_ptr(self) -> PointerValue<'tcx>;
}

impl<'tcx> LLVMAsPtrVal<'tcx> for FunctionValue<'tcx> {
    fn as_llvm_ptr(self) -> PointerValue<'tcx> {
        unsafe { std::mem::transmute(self) }
    }
}

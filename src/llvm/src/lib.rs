mod codegen;
mod codegen_ctx;
mod fcx;
mod lltypes;
mod llvm_error;
mod native;
mod tests;

use codegen::*;
pub use codegen_ctx::CodegenCtx;
pub use fcx::FnCtx;
use llvm_error::LLVMError;
use native::NativeFunctions;

use inkwell::values::{FunctionValue, PointerValue};

pub trait LLVMAsPtrVal<'tcx> {
    fn as_llvm_ptr(self) -> PointerValue<'tcx>;
}

impl<'tcx> LLVMAsPtrVal<'tcx> for FunctionValue<'tcx> {
    fn as_llvm_ptr(self) -> PointerValue<'tcx> {
        unsafe { std::mem::transmute(self) }
    }
}

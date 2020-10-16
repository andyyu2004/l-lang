#![feature(decl_macro)]
#![feature(crate_visibility_modifier)]
#![feature(array_value_iter)]

#[macro_use]
extern crate log;

mod codegen_ctx;
mod fcx;
mod intrinsics;
mod layout;
mod lltypes;
mod llvm_error;
mod monomorphize;
mod native;

#[cfg(test)]
mod tests;

pub use codegen_ctx::CodegenCtx;
pub use fcx::FnCtx;
use llvm_error::LLVMError;
use monomorphize::Monomorphize;
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

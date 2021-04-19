use crate::llvm_ty;
use inkwell::context::Context;

/// exists just to macro the work as it assumes it has a .llctx field
struct Ctx<'a> {
    llctx: &'a Context,
}

#[test]
pub fn lltype_macro_packed_struct() {
    let llctx = &Context::create();
    let expected = llctx.struct_type(&[llctx.i32_type().into(), llctx.i64_type().into()], true);
    let actual = llvm_ty!(Ctx { llctx }, packed { i32, i64 });
    assert_eq!(actual, expected);
}

#[test]
pub fn lltype_macro_struct() {
    let llctx = &Context::create();
    let expected = llctx.struct_type(&[llctx.i32_type().into(), llctx.i64_type().into()], false);
    let actual = llvm_ty!(Ctx { llctx }, {i32, i64});
    assert_eq!(actual, expected);
}

#[test]
pub fn lltype_macro_void_function() {
    let llctx = &Context::create();
    let expected =
        llctx.void_type().fn_type(&[llctx.i32_type().into(), llctx.i64_type().into()], false);
    let ctx = Ctx { llctx };
    let actual = llvm_ty!(ctx, fn(i32, i64));
    assert_eq!(actual, expected);
}

#[test]
pub fn lltype_macro_non_void_function() {
    let llctx = &Context::create();
    let expected =
        llctx.i64_type().fn_type(&[llctx.i32_type().into(), llctx.i64_type().into()], false);
    let ctx = Ctx { llctx };
    let actual = llvm_ty!(ctx, fn(i32, i64) -> i64);
    assert_eq!(actual, expected);
}
#[test]
pub fn lltype_macro_complex_function() {
    let llctx = &Context::create();
    let expected = llctx
        .struct_type(&[llctx.bool_type().into(), llctx.i32_type().into()], false)
        .fn_type(&[llctx.i32_type().into(), llctx.i64_type().into()], false);
    let ctx = Ctx { llctx };
    let actual = llvm_ty!(ctx, fn(i32, i64) -> {bool,i32});
    assert_eq!(actual, expected);
}

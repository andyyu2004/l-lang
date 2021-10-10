mod closure_tests;
mod control_flow_tests;
mod enum_tests;
mod impl_tests;
mod lltype_tests;
mod match_tests;
mod monomorphization_tests;
mod output_tests;
mod pattern_tests;
mod ptr_tests;
mod struct_tests;

pub macro llvm_jit_inner($src:expr) {
    lc_driver::Driver::from_src($src).llvm_jit()
}

pub macro llvm_jit($src:expr) {
    llvm_jit_inner!($src).unwrap()
}

pub macro llvm_jit_expect_error($src:expr) {
    llvm_jit_inner!($src).unwrap_err()
}

pub macro llvm_exec_inner($src:expr) {
    lc_driver::Driver::from_src($src).run().expect("process was interrupted before terminating")
}

/// exec *must* be called instead of jit when using a box
pub macro llvm_exec($src:expr) {
    llvm_exec_inner!($src).unwrap()
}

pub macro llvm_exec_expect_error($src:expr) {
    llvm_exec_inner!($src).unwrap_err()
}

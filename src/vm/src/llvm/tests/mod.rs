mod output_tests;
mod pattern_tests;

use crate::llvm_exec;

pub macro llvm_exec($src:expr) {
    llvm_exec($src).unwrap()
}

pub macro llvm_expect_error($src:expr) {
    llvm_exec($src).unwrap_err()
}

mod output;

use crate::llvm_exec;

pub macro llvm_exec($src:expr) {
    llvm_exec($src).unwrap()
}
mod output;

use crate::llvm_exec;

crate macro llvm_exec($src:expr) {
    llvm_exec($src).unwrap()
}

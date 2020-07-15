mod ctx;
mod heap;
#[cfg(test)]
mod tests;
mod vm;

use ctx::VMCtx;
use heap::Heap;
pub use vm::VM;

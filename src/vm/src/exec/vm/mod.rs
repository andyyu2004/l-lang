mod ctx;
mod heap;
#[cfg(test)]
mod tests;
mod vm;

use ctx::VMCtx;
pub use ctx::STACK_MAX;
use heap::Heap;
pub use vm::VM;

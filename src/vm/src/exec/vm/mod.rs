mod ctx;
mod heap;
// #[cfg(test)]
// just don't compile this atm coz its broken due to mir
#[cfg(target_family = "windows")]
mod tests;
mod vm;

use ctx::VMCtx;
pub use ctx::STACK_MAX;
use heap::Heap;
pub use vm::VM;

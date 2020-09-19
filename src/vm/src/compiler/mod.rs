//! bytecode compiler

mod compiler;
mod constant_pool;
mod ctx;
mod executable;
mod expr;
mod pat;
mod stmt;
// weird way of not compiling these test as i'm not on windows :P
#[cfg(random)]
mod tests;

pub use compiler::Compiler;
pub use constant_pool::{ConstId, Constant, ConstantPool};
pub use ctx::{FrameCtx, GlobalCompilerCtx};
pub use executable::Executable;

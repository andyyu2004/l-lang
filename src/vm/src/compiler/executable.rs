use super::ConstantPool;
use crate::exec::{CodeBuilder, Function, Op};

/// this struct defines the executable format that the vm will run, and the compiler compiles to
#[derive(Debug)]
pub struct Executable {
    pub constants: ConstantPool,
    /// the `start` function that will call the main function and begin execution
    pub start: Function,
}

impl Executable {
    // takes a function and constant pool
    // wraps the given function in code that will call it
    pub fn new(mut constants: ConstantPool, main: Function) -> Self {
        let main_index = constants.len();
        constants.push(main.into());
        let start_code = CodeBuilder::default()
            // load the given function from index `main_index`
            .emit_ldc(main_index as u8)
            // invoke it
            .emit_invoke(0)
            .emit_op(Op::ret)
            .build();

        Self { constants, start: Function::new(start_code) }
    }
}

/// execute a main function
impl From<Function> for Executable {
    /// given a function f, produces
    /// fn main() {
    ///     return f()
    /// }
    fn from(f: Function) -> Self {
        Self::new(vec![], f)
    }
}

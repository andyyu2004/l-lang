use super::ConstantPool;
use crate::{CodeBuilder, Function, Op};

/// this struct defines the executable format that the vm will run, and the compiler compiles to
#[derive(Debug)]
pub struct Executable {
    pub constants: ConstantPool,
    /// the starter function that will call main
    pub start: Function,
}

/// execute a main function
impl From<Function> for Executable {
    fn from(f: Function) -> Self {
        let start_code = CodeBuilder::default()
            // load the given function from index 0
            .emit_ldc(0)
            .emit_invoke(0)
            .emit_op(Op::ret)
            .build();

        Self {
            constants: vec![f.into()],
            start: Function::new(start_code),
        }
    }
}

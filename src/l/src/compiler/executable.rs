use super::{Constant, ConstantPool};
use crate::exec::{Code, CodeBuilder, Function, Op};
use crate::ir::DefId;
use indexed_vec::Idx;
use std::fmt::{self, Display, Formatter};

/// this struct defines the executable format that the vm will run, and the compiler compiles to
#[derive(Debug)]
pub struct Executable {
    pub constants: ConstantPool,
    /// the `start` function that will call the main function and begin execution
    pub start: Function,
}

impl Executable {
    fn mk_start_code(main_index: u8) -> Code {
        CodeBuilder::default()
            // load the given function from index `main_index`
            .emit_ldc(main_index)
            // invoke it
            .emit_invoke(0)
            .emit_op(Op::ret)
            .build()
    }

    // takes a constant pool and the index of the main function in the pool
    pub fn with_main_index(
        constants: impl IntoIterator<Item = Constant>,
        main_index: usize,
    ) -> Self {
        let constants = constants.into_iter().collect::<ConstantPool>();
        let start_code = Self::mk_start_code(main_index as u8);
        Self { constants, start: Function::new(start_code) }
    }

    // takes a main function and constant pool
    // wraps the given main function in code that will call it
    pub fn with_main(constants: impl IntoIterator<Item = Constant>, main: Function) -> Self {
        let mut constants = constants.into_iter().collect::<ConstantPool>();
        let main_index = constants.push(Constant::Function(main));
        let start_code = Self::mk_start_code(main_index.index() as u8);
        Self { constants, start: Function::new(start_code) }
    }
}

impl Display for Executable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, c) in self.constants.iter().enumerate() {
            writeln!(f, "#{}", i)?;
            writeln!(f, "{}", c)?;
        }
        Ok(())
    }
}

/// execute a main function
impl From<Function> for Executable {
    /// given a function f, produces
    /// fn main() {
    ///     return f()
    /// }
    fn from(f: Function) -> Self {
        Self::with_main(vec![], f)
    }
}
